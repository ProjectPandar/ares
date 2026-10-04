use crate::project_slice::{
    island_print_order::PreparedPostIslandPrintOrder,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod adaptive_pa;
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
mod pressure_advance;
mod processor;
mod skirt;
mod small_area;
mod spiral_vase;
mod tags;
mod temperature;
mod template;
#[cfg(test)]
mod tests;
mod timelapse;
mod timelapse_pos;
mod tool_ordering;
mod tool_transition;

#[cfg(test)]
mod timelapse_tests;
mod value;
mod wipe_tower;
use crate::{GenerationMetadata, ProcessedGCode, SliceError};

pub(super) fn emit(
    prepared: &mut PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: &mut crate::SlicerContext,
) -> Result<ProcessedGCode, SliceError> {
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
    let schedule = layers::build_schedule(
        traversal,
        &prepared.objects,
        &mut prepared.support_layers,
        &prepared.predecessor,
        context,
    )?;
    let process = &traversal.resolved.views.full.process;
    super::path_simplification::simplify_support_layers(
        &mut prepared.support_layers,
        traversal.scale,
        process.print.resolution.0,
        process.gcode.enable_arc_fitting.0 && !process.print.spiral_mode.0,
    );
    let sliced_layer_count = schedule.layer_count();
    let max_print_z = schedule.merged.last().map_or(0.0, |entry| entry.0);
    let first_layer_bounds =
        footprint::first_layer_bounds(traversal, skirt.as_ref(), brim.as_ref());
    let mut output = Vec::new();
    file_start::append(&mut output, traversal, metadata, first_layer_bounds)?;
    let tags = tags::Tags::of(traversal);
    if header::enabled(traversal) {
        header::append_header(&mut output, traversal, &schedule, max_print_z);
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
    let initial_filaments = machine::first_filaments::resolve(traversal, &schedule.tool_ordering);
    let options = motion::MotionOptions::from_traversal(traversal, 0);
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
    let extruder_offset = offset::extruder(traversal, initial_filaments.initial());
    let offset = (
        model_offset.0 - extruder_offset.0,
        model_offset.1 - extruder_offset.1,
    );
    let extrusion_role_markers = options.extrusion_role_markers;
    let extruders = motion::ExtruderRegistry::new(
        schedule.tool_ordering.all_extruders().to_vec(),
        &traversal.resolved.views.runtime_gcode,
        traversal
            .resolved
            .views
            .full
            .project
            .print
            .nozzle_diameter
            .0
            .len(),
    );
    // GCode.cpp:2707: independent PlaceholderParser context, not the fuzzy stream.
    let template_rng = std::rc::Rc::new(std::cell::RefCell::new(
        crate::slicer_context::Mt19937::new(context.seed()),
    ));
    let mut state = motion::EmitState {
        template_rng: template_rng.clone(),
        auto_lift: motion::auto_lift::AutoLift::prepare(traversal)
            .map_err(|error| SliceError::InvalidInput(format!("auto lift geometry: {error:?}")))?,
        extruders,
        initial_filaments,
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
                .then(|| {
                    placeholders::base_config(
                        traversal,
                        metadata,
                        first_layer_bounds,
                        Some(&template_rng),
                    )
                })
                .transpose()?
        },
        ..Default::default()
    };
    // Writer registration precedes machine-start and its output-variable
    // refresh/write-back (`GCode.cpp:2791,2829,3118`). No extruder is selected yet.
    machine::append_limits(&mut output, traversal);
    let bed_cache = machine::append_start(
        &mut output,
        &mut state,
        traversal,
        metadata,
        first_layer_bounds,
    )?;
    let fan_layers_start = output.len();
    let (max_layer_z, mut fan_mover_handle) = layers::append(
        prepared,
        &mut output,
        &mut state,
        layers::Context {
            metadata,
            first_layer_bounds,
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
    let volumes = finish::account_volumes(&output, &filament.filament_diameter.0);
    let used_filament = volumes
        .iter()
        .zip(&filament.filament_diameter.0)
        .map(|(volume, diameter)| {
            let diameter = f64::from(diameter.0 as f32);
            volume / (std::f64::consts::PI * (0.5 * diameter).powi(2))
        })
        .sum();
    let (total_weight, total_cost) =
        finish::append_filament_stats(&mut output, traversal, &volumes, &state);
    finish::append_compatible_stats(
        &mut output,
        traversal,
        (total_weight, total_cost),
        tail_layer_count,
        state.initial_filaments.total_toolchanges(),
    );
    output.push(b'\n');
    // Upstream applies FanMover ONLY to layer-chunk output (the tbb
    // pipeline after the cooling filter, GCode.cpp:3749), never to the
    // machine start g-code.
    if fan_mover_handle.is_some()
        || state
            .extruders
            .ids()
            .iter()
            .any(|&tool| adaptive_pa::enabled(&traversal.resolved.views.full, tool))
    {
        let mut layers = output.split_off(fan_layers_start);
        let tail = layers.split_off(fan_layers_end - fan_layers_start);
        if let Some(mover) = fan_mover_handle.as_mut() {
            layers = fan_chunks::process_layer_chunks(mover, layers);
        }
        output.extend(adaptive_pa::process(layers, traversal, &state));
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
        processor::ProcessorLimits::from_traversal(traversal, state.tags.is_bbl())
            .with_settings(&traversal.resolved.views.full)
            .recording_moves(record_moves),
    ))
}

pub(in crate::project_slice) use emit_helpers::format_processor_float;
