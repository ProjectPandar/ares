use crate::project_slice::{
    extruders, island_print_order::PreparedPostIslandPrintOrder,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod brim;
mod emit_helpers;
use emit_helpers::{append_layer_end_timelapse, layer_boundary_slices_rc, trailing_gcode_xy};
mod cooling;
mod expression;
mod fan_chunks;
mod fan_mover;
mod file_start;
mod finish;
pub(super) mod footprint;
mod header;
mod layer_gcode;
mod layers;
mod lexer;
mod machine;
pub(in crate::project_slice) mod motion;
mod object;
mod offset;
mod placeholders;
mod processor;
mod skirt;
mod small_area;
mod spiral_vase;
mod tags;
mod template;
#[cfg(test)]
mod tests;
mod timelapse;
mod timelapse_pos;

#[cfg(test)]
mod timelapse_tests;
mod value;
use crate::{GenerationMetadata, SliceError};

pub(super) fn emit(
    prepared: &mut PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
    context: &mut crate::SlicerContext,
) -> Result<Vec<u8>, SliceError> {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let brim = brim::BrimPlan::generate(traversal)?;
    let skirt =
        skirt::SkirtPlan::generate(traversal, &prepared.support_layers, brim.as_ref(), context)?;
    let process = &traversal.resolved.views.full.process;
    super::path_simplification::simplify_support_layers(
        &mut prepared.support_layers,
        traversal.scale,
        process.print.resolution.0,
        process.gcode.enable_arc_fitting.0 && !process.print.spiral_mode.0,
    );
    let schedule = layers::build_schedule(
        traversal,
        &prepared.objects,
        &prepared.support_layers,
        context,
    )?;
    let sliced_layer_count = schedule.layer_count();
    let max_print_z = schedule.merged.last().map_or(0.0, |entry| entry.0);
    let first_layer_bounds =
        footprint::first_layer_bounds(traversal, skirt.as_ref(), brim.as_ref());
    let mut output = Vec::new();
    file_start::append(&mut output, traversal, metadata, first_layer_bounds)?;
    let tags = tags::Tags::of(traversal);
    if header::enabled(traversal) {
        header::append_header(
            &mut output,
            metadata,
            traversal,
            sliced_layer_count,
            max_print_z,
        );
    }
    // `GCode.cpp` + Orca export layout: BBL keeps the config block up front;
    // the compatible flavor moves it after the tail statistics.
    if tags.is_bbl()
        && let Some(config) = &traversal.config_block
    {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal)?;
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    object::append_definitions(&mut output, traversal);
    machine::append_first_line_m73(&mut output);
    // The Marlin-family machine envelope prints before the start G-code
    // (`GCode.cpp:2819`), followed by the start G-code (`GCode.cpp:3137`).
    machine::append_limits(&mut output, traversal);
    let start_outcome =
        machine::append_start(&mut output, traversal, metadata, first_layer_bounds)?;
    let (bed_cache, start_position, start_retract, start_failure) = (
        start_outcome.bed_cache,
        start_outcome.position,
        start_outcome.retract,
        start_outcome.failure,
    );
    let options = motion::MotionOptions::from_traversal(traversal);
    let small_area_flow = small_area::from_traversal(traversal)?;
    let model_offset = traversal
        .resolved
        .objects
        .first()
        .map_or((0.0, 0.0), |object| {
            footprint::object_origin(
                &traversal.project.objects()[object.source_object_index],
                traversal.scale,
            )
        });
    let extruder_offset = offset::initial_extruder(traversal);
    let offset = (
        model_offset.0 - extruder_offset.0,
        model_offset.1 - extruder_offset.1,
    );
    let extrusion_role_markers = options.extrusion_role_markers;
    let mut state = motion::EmitState {
        offset,
        origin: model_offset,
        extruder_offset,
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        small_area_flow,
        tags: tags::Tags::of(traversal),
        spiral_vase: traversal.resolved.views.full.process.print.spiral_mode.0,
        extrusion_role_markers,
        role_change_config: {
            let runtime = &traversal.resolved.views.runtime_gcode;
            let needed = !runtime.change_extrusion_role_gcode.0.is_empty()
                || !runtime.process_change_extrusion_role_gcode.0.is_empty()
                || runtime
                    .filament_change_extrusion_role_gcode
                    .0
                    .iter()
                    .any(|template| !template.is_empty());
            needed
                .then(|| placeholders::base_config(traversal, metadata, first_layer_bounds))
                .transpose()?
        },
        ..Default::default()
    };
    if let Some(error) = start_failure {
        state.record_template_failure("machine_start_gcode", &error);
    }
    // `e_position[0]` write-back — absolute-E counters feed the
    // filament-start `{e_position}` placeholder (`GCode.cpp:3908-3911`;
    // sub-EPSILON assignments are ignored via `is_approx`).
    if let Some(e_position) = start_outcome.e_position {
        // `set_position` writes the private `m_E` and the compare reads
        // `position()` (`Extruder.hpp:49,56`; `GCode.cpp:3910-3911`).
        let semm = state.options.single_extruder_multi_material;
        let current = if semm {
            state.private_e_position
        } else {
            state.e_position
        };
        if (e_position - current).abs() >= 1e-4 {
            if semm {
                state.private_e_position = e_position;
            } else {
                state.e_position = e_position;
            }
        }
    }
    // The machine-start template's `e_retracted[0]` assignment seeds the
    // extruder state (`GCode.cpp:3905-3918`): a pre-retracted nozzle skips
    // the layer-change retract and the first unretract restores exactly the
    // assigned amount.
    if let Some(retract) = &start_retract {
        // `Extruder::set_retracted` (`Extruder.cpp:108-122`): values below
        // `-EPSILON` throw (upstream aborts the export after writing the
        // inline failure block — `GCode.cpp:3920-3933`); a retracted
        // > EPSILON assignment takes both fields, otherwise both clear.
        const EPSILON: f64 = 1e-4;
        if retract.retracted < -EPSILON {
            state.record_template_failure(
                "machine_start_gcode",
                "Custom G-code reports negative z_retracted.",
            );
        } else if retract.restart_extra < -EPSILON {
            state.record_template_failure(
                "machine_start_gcode",
                "Custom G-code reports negative z_restart_extra.",
            );
        } else if retract.retracted > EPSILON {
            // `set_retracted` targets the PRIVATE `m_retracted`
            // (`Extruder.cpp:108-122`); under SEMM only the placeholder
            // echoes observe it. `m_restart_extra` is the same field in
            // both modes — the shared `unretract` consumes it
            // (`Extruder.cpp:87-90,117`).
            let restart_extra = if retract.restart_extra < EPSILON {
                0.0
            } else {
                retract.restart_extra
            };
            if state.options.single_extruder_multi_material {
                state.private_retracted = retract.retracted;
            } else {
                state.retracted = true;
                state.retracted_amount = retract.retracted;
            }
            state.restart_extra = restart_extra;
        } else if state.options.single_extruder_multi_material {
            // `retracted <= EPSILON` clears BOTH fields
            // (`Extruder.cpp:118-121`).
            state.private_retracted = 0.0;
            state.restart_extra = 0.0;
        } else {
            state.retracted = false;
            state.retracted_amount = 0.0;
            state.restart_extra = 0.0;
        }
    }
    // A `position` assignment in the machine-start template writes back
    // into the writer state — including the generator cursor — for every
    // template call (`GCode.cpp:3900-3907`, `Point.hpp:665` truncates).
    // It is overwritten again by the silent `travel_to_z(z_offset)` in
    // `preamble` (`GCode.cpp:5676-5679`, applied inside
    // `preamble::emit`). `set_last_pos` runs only when the position
    // actually changed from the writer's initial (0,0,0)
    // (`GCode.cpp:3900-3903`).
    if let Some(value::Value::List(position)) = &start_position
        && let [x, y, z] = position.as_slice()
        && let (Some(x), Some(y), Some(z)) = (x.as_number(), y.as_number(), z.as_number())
    {
        let moved = x != 0.0 || y != 0.0 || z != 0.0;
        state.x = x;
        state.y = y;
        state.writer_z = Some(z);
        if moved {
            // `gcode_to_point` adds the extruder offset only with an
            // active extruder (`GCode.cpp:8138-8144`); machine-start
            // runs before the initial `set_extruder`
            // (`GCode.cpp:3144`), so no offset applies here.
            state.last_scaled_position = Some((
                ((x - state.origin.0) / state.scale_factor) as i64,
                ((y - state.origin.1) / state.scale_factor) as i64,
            ));
        }
    }
    let fan_layers_start = output.len();
    let (max_layer_z, mut fan_mover_handle) = layers::append(
        prepared,
        &mut output,
        &mut state,
        layers::Context {
            metadata,
            first_layer_bounds,
            start_position,
            bed_cache,
            brim: &brim,
            skirt: &skirt,
            schedule,
        },
    )?;
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let fan_layers_end = output.len();
    // The compatible tail's `; total layers count` prints `m_layer_count`
    // (`GCode.cpp:3542`) — the sliced layer count, NOT the processor's
    // tag-based count the header placeholder carries.
    let tail_layer_count = sliced_layer_count;
    // The final compatible layer has no following layer marker to flush its
    // deferred retraction. Flush only retract/wipe (not a travel lift) before
    // end G-code (`GCode.cpp` final object teardown).
    motion::retract_for_print_end(&mut output, &mut state);
    motion::append_exclude_end(&mut output, &mut state);
    // `position` = the LIVE writer position at print end
    // (`GCode.cpp:3478`), including a trailing lift/timelapse Z — not
    // the nominal layer z.
    let end_position = (state.x, state.y, state.writer_z.unwrap_or(state.layer_z));
    finish::append(
        &mut output,
        &mut state,
        traversal,
        max_layer_z,
        metadata,
        first_layer_bounds,
        end_position,
    )?;
    machine::append_completion_controls(&mut output, traversal);
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    let filament = &traversal.resolved.views.full.filament.gcode;
    let used_filament = finish::account_used_filament(&output, filament.filament_diameter.0[0].0);
    let (total_weight, total_cost) = finish::append_filament_stats(
        &mut output,
        traversal,
        used_filament,
        state.extrusion_totals().used_filament,
    );
    finish::append_compatible_stats(
        &mut output,
        traversal,
        total_weight,
        total_cost,
        tail_layer_count,
    );
    output.push(b'\n');
    // Upstream applies FanMover ONLY to layer-chunk output (the tbb
    // pipeline after the cooling filter, GCode.cpp:3749), never to the
    // machine start g-code.
    if let Some(mover) = fan_mover_handle.as_mut() {
        let mut layers = output.split_off(fan_layers_start);
        let tail = layers.split_off(fan_layers_end - fan_layers_start);
        let processed = fan_chunks::process_layer_chunks(mover, layers);
        output.extend(processed);
        output.extend(tail);
    }
    // Upstream throws after emit when any custom template failed
    // (`GCode.cpp:3637-3645` — `failed_templates` collected via
    // `placeholder_parser_process` `GCode.cpp:3920-3933`).
    if let Some((name, error)) = state.failed_templates.first() {
        return Err(SliceError::InvalidInput(format!(
            "failed to process the custom G-code template {name}: {error}"
        )));
    }
    // `GCodeProcessor.cpp:2920-2926,3323-3325,1136-1139`: count exact
    // reserved comment lines in the complete stream, including end templates.
    if header::enabled(traversal) {
        let layer_count = output
            .split(|byte| *byte == b'\n')
            .filter(|line| {
                line.strip_suffix(b"\r").unwrap_or(line) == state.tags.layer_change().as_bytes()
            })
            .count();
        header::finalize_layer_count(&mut output, layer_count);
    }
    Ok(processor::process(
        output,
        !traversal.resolved.views.full.printer.gcode.disable_m73.0,
        traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .machine_load_filament_time
            .0,
        used_filament,
        processor::ProcessorLimits::from_traversal(traversal, state.tags.is_bbl()),
    ))
}

pub(in crate::project_slice) use emit_helpers::format_processor_float;
