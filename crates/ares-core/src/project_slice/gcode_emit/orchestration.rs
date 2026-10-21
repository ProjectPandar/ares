use super::*;
use crate::project_slice::island_print_order::PreparedPostIslandPrintOrder;
use crate::{GenerationMetadata, ProcessedGCode, SliceError};

pub(in crate::project_slice) fn emit(
    prepared: &mut PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: &mut crate::SlicerContext,
) -> Result<ProcessedGCode, SliceError> {
    let mut schedule = layers::build_schedule(prepared, context)?;
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let tower_exclusions = schedule.tower.as_ref().map_or_else(Vec::new, |tower| {
        tower.brim_exclusions(&traversal.resolved.views.full, traversal.scale)
    });
    let mut brim = brim::BrimPlan::generate_all(
        traversal,
        &prepared.support_layers,
        &tower_exclusions,
        |index| &prepared.objects[index][0].tool_regions,
    )?;
    brim::combine(traversal, &mut brim)?;
    let tower_corners = schedule.tower.as_ref().map_or_else(Vec::new, |tower| {
        tower.first_layer_corners(&traversal.resolved.views.full, traversal.scale)
    });
    let skirt = skirt::SkirtPlan::generate(
        traversal,
        &prepared.support_layers,
        &brim,
        (&tower_corners, schedule.tool_ordering.all_extruders()),
        schedule.raft.as_ref(),
    )?;
    let islands =
        footprint::FirstLayerIslands::collect(traversal, &prepared.support_layers, &schedule)?;
    let process = &traversal.resolved.views.full.process;
    crate::project_slice::path_simplification::simplify_support_layers(
        &mut prepared.support_layers,
        traversal.scale,
        process.print.resolution.0,
        process.gcode.enable_arc_fitting.0 && !process.print.spiral_mode.0,
    );
    schedule.simplify_raft(traversal);
    conflict::check(
        traversal,
        &prepared.objects,
        &prepared.support_layers,
        &schedule,
    )?;
    let sliced_layer_count = schedule.layer_count();
    // Under sequential by-object printing `m_layer_count` accumulates the
    // per-object layer counts (`GCode.cpp:2505`, `+= instances.size() *
    // temp_layer_count` in the sequential branch); the by-layer branch
    // (:2522) counts merged print-z across all objects once. Either way the
    // compatible tail (`GCode.cpp:3542`) prints the print-wide total, not a
    // single object's layer count.
    // `m_layer_count` is mode-dependent (`GCode.cpp:2487-2528`): the
    // sequential by-object branch accumulates each object's layer count
    // per instance (:2505), the by-layer branch counts merged print-z
    // across all objects once (:2522). The compatible tail (:3542) and
    // the `total_layer_count` placeholder (:2890) both print it.
    let by_object = traversal.resolved.views.full.process.print.print_sequence
        == crate::ProcessPrintSequence::ByObject;
    let tail_layer_count = if by_object {
        schedule.m73_layer_count().max(sliced_layer_count)
    } else {
        sliced_layer_count
    };
    let max_print_z = schedule.merged.last().map_or(0.0, |entry| entry.0);
    let first_layer_bounds = footprint::first_layer_bounds(
        traversal,
        skirt.as_ref(),
        &brim,
        &tower_corners,
        &islands.plate,
    );
    let mut output = Vec::new();
    file_start::append(
        &mut output,
        traversal,
        metadata,
        first_layer_bounds,
        sliced_layer_count,
    )?;
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
    // `GCode.hpp:185-187`: origin starts at zero; real set_origin calls
    // accumulate the numeric generator cursor's local-frame translations.
    let origin = (0.0, 0.0);
    let extruder_offset = offset::extruder(traversal, initial_filaments.initial());
    let offset = (origin.0 - extruder_offset.0, origin.1 - extruder_offset.1);
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
        total_layer_count: sliced_layer_count,
        template_rng: template_rng.clone(),
        first_layer_hull: footprint::first_layer_convex_hull(
            traversal,
            skirt.as_ref(),
            &brim,
            &tower_corners,
            &islands.plate,
        ),
        hold_chamber_temp_for_flat_print: chamber::hold_temperature(
            traversal,
            &prepared.support_layers,
            &schedule,
            &brim,
        )?,
        wipe_tower: schedule.tower.take().map(|data| {
            let settings = &traversal.resolved.views.full;
            let first_z = schedule.merged[0].0;
            let second_z = schedule
                .merged
                .iter()
                .map(|entry| entry.0)
                .find(|&z| z > first_z)
                .unwrap_or(first_z);
            let overlap = settings
                .process
                .gcode
                .single_extruder_multi_material_priming
                .0
                && data.priming_overlaps(wipe_tower::PrintExtents {
                    traversal,
                    objects: &prepared.objects,
                    instances: &schedule.instances,
                    skirt: skirt.as_ref(),
                    max_print_z: second_z + 1e-4,
                });
            let center = footprint::first_layer_object_center(
                traversal,
                &schedule.instances,
                &brim,
                &islands.support_spacing,
            );
            wipe_tower::Integration::new(data, settings, center, overlap)
        }),
        auto_lift: motion::auto_lift::AutoLift::prepare(traversal)
            .map_err(|error| SliceError::InvalidInput(format!("auto lift geometry: {error:?}")))?,
        extruders,
        initial_filaments,
        offset,
        origin,
        extruder_offset,
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: 3600.0,
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
                        sliced_layer_count,
                    )
                })
                .transpose()?
        },
        ..Default::default()
    };
    // Writer registration precedes machine-start and its output-variable
    // refresh/write-back (`GCode.cpp:2791,2829,3118`). No extruder is selected yet.
    machine::append_limits(&mut output, traversal, state.initial_filaments.initial());
    let bed_cache = machine::append_start(
        &mut output,
        &mut state,
        traversal,
        metadata,
        first_layer_bounds,
    )?;
    let fan_layers_start = output.len();
    let (max_layer_z, mut fan_mover_handle, fan_layers_end) = layers::append(
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
    if state.contour_z_too_low {
        // GCode.cpp:7025/7233, OrcaSlicer.cpp:6286-6291: export failure, not config admission.
        let settings = &traversal.resolved.views.full;
        return Err(SliceError::Slicing {
            message: "GCode: very low z".to_owned(),
            plate_index: traversal.project.first_plate_id(),
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        });
    }

    // The compatible tail's `; total layers count` prints `m_layer_count`
    // (`GCode.cpp:3542`) — the sliced layer count, NOT the processor's
    // tag-based count the header placeholder carries.
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
        finish::FinishContext {
            metadata,
            first_layer_bounds,
            position: end_position,
        },
    )?;
    machine::append_completion_controls(&mut output, traversal, &state);
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    let filament = &traversal.resolved.views.full.filament.gcode;
    // `GCodeProcessorResult::extruder_colors`: `filament_colour` padded
    // to `filaments_count`, missing/empty slots → `#FF8000`
    // (`GCodeProcessor::apply_config`, `GCodeProcessor.cpp:2321-2343`).
    let extruder_colors = (0..filament.filament_diameter.0.len())
        .map(|id| {
            filament
                .filament_colour
                .0
                .get(id)
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| "#FF8000".to_string())
        })
        .collect::<Vec<_>>();
    finish::append_filament_stats(&mut output, traversal, &state);
    finish::append_compatible_stats(
        &mut output,
        traversal,
        tail_layer_count,
        state.wipe_tower.as_ref().map_or_else(
            || state.initial_filaments.total_toolchanges(),
            wipe_tower::Integration::tool_changes,
        ),
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
            layers = fan_chunks::process_layer_chunks(mover, layers, state.tags.layer_change());
        }
        output.extend(adaptive_pa::process(layers, traversal, &state));
        output.extend(tail);
    }
    // Upstream throws after emit when any custom template failed
    // (`GCode.cpp:3637-3645` — `failed_templates` collected via
    // `placeholder_parser_process` `GCode.cpp:3920-3933`).
    if !state.failed_templates.is_empty() {
        return Err(template_failure::from_failed_templates(
            &state.failed_templates,
            traversal,
        ));
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
    let (processed, printable_area) = processor::process(
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
        processor::ProcessorLimits::from_traversal(traversal, state.tags.is_bbl())
            .with_settings(&traversal.resolved.views.full)
            .extruder_colors(extruder_colors)
            .writer_extruders(state.extruders.ids().to_vec())
            .recording_moves(record_moves),
    );
    retained_output::check(processed, printable_area, traversal)
}
