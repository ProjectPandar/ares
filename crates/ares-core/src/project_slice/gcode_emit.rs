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
    let skirt = skirt::SkirtPlan::generate(traversal, brim.as_ref())?;
    let first_layer_bounds =
        footprint::first_layer_bounds(traversal, skirt.as_ref(), brim.as_ref());
    let mut output = Vec::new();
    file_start::append(&mut output, traversal, metadata, first_layer_bounds)?;
    let tags = tags::Tags::of(traversal);
    header::append_header(&mut output, metadata, traversal);
    // `GCode.cpp` + Orca export layout: BBL keeps the config block up front;
    // the compatible flavor moves it after the tail statistics.
    if tags.is_bbl()
        && let Some(config) = &traversal.config_block
    {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
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
    let model_offset = footprint::model_center(traversal).unwrap_or_default();
    let model_offset = (
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.0).unwrap()),
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.1).unwrap()),
    );
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
    if let Some(e_position) = start_outcome.e_position
        && (e_position - state.e_position).abs() >= 1e-4
    {
        state.e_position = e_position;
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
            state.retracted = true;
            state.retracted_amount = retract.retracted;
            state.restart_extra = if retract.restart_extra < EPSILON {
                0.0
            } else {
                retract.restart_extra
            };
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
            state.last_scaled_position = Some((
                ((x - state.origin.0 + state.extruder_offset.0) / state.scale_factor) as i64,
                ((y - state.origin.1 + state.extruder_offset.1) / state.scale_factor) as i64,
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
            extruder_offset,
            brim: &brim,
            skirt: &skirt,
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
    // `GCodeProcessor.cpp:2139-2142`: in spiral vase mode the processor
    // counts layers from the `;LAYER_CHANGE` tags in the emitted stream
    // (`m_detect_layer_based_on_tag`), so the transition_out pseudo-layer
    // (its duplicated header) counts as a layer.
    let layer_count = if traversal.resolved.views.full.process.print.spiral_mode.0 {
        output
            .windows(header::LAYER_CHANGE_TAG.len())
            .filter(|window| *window == header::LAYER_CHANGE_TAG)
            .count()
    } else {
        header::plate_layer_count(traversal)
    };
    let emitted_layer_count = header::finalize_layer_count(&mut output, layer_count);
    // The compatible tail's `; total layers count` prints `m_layer_count`
    // (`GCode.cpp:3542`) — the sliced layer count, NOT the processor's
    // tag-based count the header placeholder carries.
    let tail_layer_count = header::plate_layer_count(traversal);
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
    let used_filament = finish::account_used_filament(&output);
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
        processor::ProcessorLimits {
            retraction_length: traversal
                .resolved
                .views
                .runtime_gcode
                .retraction_length
                .0
                .first()
                .map_or(0.0, |value| value.0),
            retract_restart_extra: traversal
                .resolved
                .views
                .runtime_gcode
                .retract_restart_extra
                .0
                .first()
                .map_or(0.0, |value| value.0),
            retraction_feedrate: traversal
                .resolved
                .views
                .runtime_gcode
                .retraction_speed
                .0
                .first()
                .map_or(0.0, |value| value.0)
                * 60.0,
            deretraction_feedrate: {
                let retraction = traversal
                    .resolved
                    .views
                    .runtime_gcode
                    .retraction_speed
                    .0
                    .first()
                    .map_or(0.0, |value| value.0);
                let deretraction = traversal
                    .resolved
                    .views
                    .runtime_gcode
                    .deretraction_speed
                    .0
                    .first()
                    .map_or(0.0, |value| value.0);
                (if deretraction > 0.0 {
                    deretraction
                } else {
                    retraction
                }) * 60.0
            },
            ..processor::ProcessorLimits::from_config(
                &traversal.resolved.views.full.printer.machine,
                traversal.resolved.views.full.printer.gcode.gcode_flavor,
                state.tags.is_bbl(),
            )
        },
    ))
}

pub(super) fn format_processor_float(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let precision = (5 - magnitude).max(0) as usize;
    let mut formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }
    formatted
}
