mod state;
mod tower;

use super::*;
use crate::project_slice::ArchiveSource;
use crate::project_slice::island_print_order::PreparedPostIslandPrintOrder;
use crate::{GCodeSink, GenerationMetadata, SliceError, StreamedGCode};

pub(in crate::project_slice) fn emit_to_sink(
    prepared: &mut PreparedPostIslandPrintOrder,
    (metadata, record_moves): (GenerationMetadata, bool),
    (context, sink): (&mut crate::SlicerContext, &mut dyn GCodeSink),
    warnings: &mut Vec<crate::PrintWarning>,
    archive: Option<ArchiveSource>,
) -> Result<StreamedGCode, SliceError> {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let instances = layers::instances::collect(traversal);
    let farthest_object_targets =
        layers::farthest::ObjectTargets::capture(traversal, &prepared.objects, &instances);
    crate::project_slice::seam_placement::apply(prepared, context.cancel_ref())?;
    // `Print.cpp:3540-3550` detects auto-lift overhangs per object (`PrintObject.cpp:940`; Ares runs
    // the detection in `AutoLift::prepare`), then makes the skirt and brim (`Print.cpp:3638`).
    for _ in &prepared.objects {
        context.set_status(71, format_args!("Detect overhangs for auto-lift"), 0);
    }
    context.set_status(70, format_args!("Generating skirt & brim"), 0);
    let mut schedule = layers::build_schedule(prepared, instances)?;
    warnings.append(&mut schedule.warnings);
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let tower_exclusions = schedule.tower.as_ref().map_or_else(Vec::new, |tower| {
        tower.brim_exclusions(wipe_tower::plate_position(traversal), traversal.scale)
    });
    let brim = brim::BrimPlan::generate_all(
        traversal,
        &prepared.support_layers,
        &tower_exclusions,
        |index| &prepared.objects[index][0].tool_regions,
    )?;
    let archive_facts = archive
        .map(|source| archive::facts(traversal, &schedule, &brim, source))
        .transpose()?;
    let tower_corners = schedule.tower.as_ref().map_or_else(Vec::new, |tower| {
        let [x, y] = wipe_tower::plate_position(traversal);
        let origin = traversal.project.plate_origin();
        tower.first_layer_corners(
            &traversal.resolved.views.full,
            traversal.scale,
            [x + origin.0, y + origin.1],
        )
    });
    let skirt = skirt::SkirtPlan::generate(
        traversal,
        &prepared.support_layers,
        &brim,
        &tower_corners,
        schedule.raft.as_ref(),
    )?;
    let islands =
        footprint::FirstLayerIslands::collect(traversal, &prepared.support_layers, &schedule)?;
    // `PrintObject.cpp:1061,1079,1097`: three per object; walls and infill ran in `path_simplification`.
    for _ in 0..3 * prepared.objects.len() {
        context.set_status(75, format_args!("Optimizing toolpath"), 0);
    }
    let process = &traversal.resolved.views.full.process;
    crate::project_slice::path_simplification::simplify_support_layers(
        &mut prepared.support_layers,
        traversal.scale,
        process.print.resolution.0,
        process.gcode.enable_arc_fitting == crate::ArcFittingMode::Enabled
            && !process.print.spiral_mode.0,
    );
    schedule.simplify_raft(traversal);
    conflict::check_print(
        traversal,
        &prepared.objects,
        &prepared.support_layers,
        &schedule,
        context,
    )?;
    context.set_status(80, format_args!("Generating G-code"), 0);
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
    let first_layer_bounds = footprint::first_layer_bounds(
        traversal,
        skirt.as_ref(),
        &brim,
        &tower_corners,
        &islands.plate,
    );
    let initial_filaments = machine::first_filaments::resolve(traversal, &schedule.tool_ordering);
    let nozzle_groups = std::mem::take(&mut schedule.tool_ordering.nozzle_groups);
    let compat_temperatures = initial_filaments.compat_temperatures(traversal);
    let export_config = if !matches!(
        (archive, &archive_facts),
        (Some(ArchiveSource::Saved), Some(_))
    ) && let Some(base) = traversal.config_block.as_deref()
        && let Some(patched) = crate::options::config_block_with_compat_temperatures(
            base,
            &traversal.resolved.views,
            compat_temperatures,
        ) {
        patched
    } else {
        let mut export_config = Vec::new();
        crate::options::write_config_block(
            &traversal.resolved.views,
            (
                match (archive, &archive_facts) {
                    (Some(ArchiveSource::Saved), Some(facts)) => &facts.project_settings,
                    _ => &traversal.project.documents().project_settings_raw,
                },
                traversal.project.documents().config_composer,
            ),
            traversal.project.current_plate_index(),
            Some(compat_temperatures),
            &mut export_config,
        )?;
        export_config
    };
    let mut state = state::build(state::EmitStateInputs {
        traversal,
        prepared,
        schedule: &mut schedule,
        metadata,
        seed: context.seed(),
        brim: &brim,
        skirt: skirt.as_ref(),
        tower_corners: &tower_corners,
        islands: &islands,
        first_layer_bounds,
        initial_filaments,
        sliced_layer_count,
        tail_layer_count,
    })?;
    let mut output = Vec::new();
    let mode_gcode = file_start::append(
        &mut output,
        file_start::FileStart {
            traversal,
            metadata,
            first_layer_bounds,
            layer_count: sliced_layer_count,
        },
        &mut state,
    )?;
    let tags = preamble::append(
        &mut output,
        traversal,
        &schedule,
        &export_config,
        context.thumbnail_images(),
    )?;
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    object::append_definitions(&mut output, traversal);
    machine::append_first_line_m73(&mut output);
    // Writer registration precedes machine-start and its output-variable
    // refresh/write-back (`GCode.cpp:2791,2829,3118`). No extruder is selected yet.
    machine::append_limits(
        &mut output,
        traversal,
        state.initial_filaments.initial(),
        state.extruders.ids(),
    );
    let bed_cache = machine::append_start(
        &mut output,
        &mut state,
        traversal,
        metadata,
        (first_layer_bounds, &mode_gcode),
    )?;
    let filtered_layers_start = output.len();
    let (max_layer_z, mut fan_mover_handle, fan_layers_end, fan_layers_start) = layers::append(
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
            farthest_object_targets: farthest_object_targets.as_ref(),
        },
        context,
    )?;
    // Release consumed per-layer geometry before post-processing and streaming.
    prepared.objects = Vec::new();
    prepared.support_layers = Vec::new();
    prepared.top_surfaces = Vec::new();
    prepared.curled_lines = Vec::new();
    prepared.nearest_seam_plans = Vec::new();
    drop((farthest_object_targets, brim, skirt, tower_corners, islands));
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
    if !tags.is_bbl() {
        output.extend_from_slice(&export_config);
    }
    drop(export_config);
    let tower_statistics = finish::append_filament_stats(&mut output, traversal, &state);
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
    // Initial preamble bypasses FanMover (`GCode.cpp:4495,5192–5220`).
    let run_adaptive_pa = !traversal.resolved.views.full.process.print.spiral_mode.0
        && state
            .extruders
            .ids()
            .iter()
            .any(|&tool| adaptive_pa::enabled(&traversal.resolved.views.full, tool));
    match (fan_mover_handle.as_mut(), run_adaptive_pa) {
        (None, false) => {}
        (Some(mover), false) => {
            let mut rebuilt = Vec::with_capacity(output.len());
            rebuilt.extend_from_slice(&output[..fan_layers_start]);
            fan_chunks::process_layer_chunks_into(
                mover,
                &output[fan_layers_start..fan_layers_end],
                state.tags.layer_change(),
                &mut rebuilt,
            );
            rebuilt.extend_from_slice(&output[fan_layers_end..]);
            output = rebuilt;
        }
        (None, true) => {
            let mut rebuilt = Vec::with_capacity(output.len());
            rebuilt.extend_from_slice(&output[..filtered_layers_start]);
            adaptive_pa::process_into(
                &output[filtered_layers_start..fan_layers_end],
                traversal,
                &state,
                &mut rebuilt,
            );
            rebuilt.extend_from_slice(&output[fan_layers_end..]);
            output = rebuilt;
        }
        (Some(mover), true) => {
            let mut fan_layers = Vec::with_capacity(fan_layers_end - filtered_layers_start);
            fan_layers.extend_from_slice(&output[filtered_layers_start..fan_layers_start]);
            fan_chunks::process_layer_chunks_into(
                mover,
                &output[fan_layers_start..fan_layers_end],
                state.tags.layer_change(),
                &mut fan_layers,
            );
            let mut rebuilt = Vec::with_capacity(output.len());
            rebuilt.extend_from_slice(&output[..filtered_layers_start]);
            adaptive_pa::process_into(&fan_layers, traversal, &state, &mut rebuilt);
            rebuilt.extend_from_slice(&output[fan_layers_end..]);
            output = rebuilt;
        }
    }
    let mut result = super::orchestration_finish::finish_to_sink(
        output,
        &state,
        traversal,
        (&nozzle_groups, (record_moves, tower_statistics)),
        sink,
    )?;
    result.archive = archive_facts.map(archive::sliced_archive).transpose()?;
    Ok(result)
}
