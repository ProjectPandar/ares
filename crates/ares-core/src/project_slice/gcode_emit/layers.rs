//! GCode.cpp layer-chunk emission, extracted without changing command ordering.
mod boundary;
mod brim_emit;
mod context;
mod cooling_result;
mod custom_gcode;
mod entry;
mod entry_builder;
mod fan_setup;
mod group;
mod layer_end;

pub(super) use context::Context;
mod instances;
mod island_visits;
pub(super) use instances::Instance;
mod object_entry;
pub(super) mod object_order;
mod object_skirt;
mod preamble;
mod raft_entry;
mod raft_schedule;
mod schedule;
mod skirt_emit;
mod skirt_schedule;
mod support_entry;
mod timelapse;
mod tool_buckets;
mod tool_entry;
mod tower;
mod transition;
mod wrapping;

use entry_builder::EntryBuilder;
pub(super) use raft_schedule::RaftSchedule;
pub(super) use schedule::Schedule;
pub(super) use schedule::build as build_schedule;

use super::{
    GenerationMetadata, PreparedPostIslandPrintOrder, SliceError, cooling, fan_mover, layer_gcode,
    motion, object, spiral_vase,
};

pub(super) fn append(
    prepared: &mut PreparedPostIslandPrintOrder,
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: Context<'_>,
) -> Result<(f64, Option<fan_mover::FanMover>, usize, usize), SliceError> {
    let Context {
        metadata,
        first_layer_bounds,
        mut bed_cache,
        brim,
        skirt,
        schedule,
    } = context;
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    // Disjoint field borrows: `objects` stays mutable while the geometry
    // buffers (top surfaces, seam plans, the traversal chain) are shared.
    let objects = &mut prepared.objects;
    let support_layers = &mut prepared.support_layers;
    let top_surfaces = &prepared.top_surfaces;
    let curled_lines = &prepared.curled_lines;
    let nearest_seam_plans = &prepared.nearest_seam_plans;
    let island_predecessor = &prepared.predecessor;
    let emit_labels = traversal
        .resolved
        .views
        .full
        .process
        .print
        .gcode_label_objects
        .0;
    let mut object_skirt_done = vec![(0, 0.0); skirt.as_ref().map_or(0, |plan| plan.group_count())];
    let mut cooling = cooling::CoolingState::from_traversal(traversal, state);
    let mut spiral =
        spiral_vase::SpiralVaseFilter::from_traversal(traversal, brim.iter().any(Option::is_some));
    // `GCode.cpp:4629-4630`: upstream tracks `m_max_layer_z` as the
    // running max of `float(print_z)` — the f32 cast absorbs the f64
    // accumulation drift so `{max_layer_z}`/`max_print_z` placeholders
    // read the float-domain top layer, not a raw f64 height sum.
    let mut max_layer_z = 0.0f64;
    let layer_change_template =
        layer_gcode::LayerChangeTemplate::new(traversal, metadata, first_layer_bounds);
    let timelapse_gate = timelapse::gate(traversal, state);
    // `ToolOrdering::cal_most_used_extruder` (`ToolOrdering.cpp:955-981`) —
    // consumed by the toolchange-boundary timelapse insert gate
    // (`GCode.cpp:5292-5297`).
    let traditional_interlude = timelapse_gate.traditional_interlude;
    let mut second_layer_done = false;
    // Avoid-crossing boundaries are built per slice_z across every object
    // (`Layer::lslices` covers all instances of the print object;
    // `AvoidCrossingPerimeters.cpp:1100`). Copies of one source share the
    // layer layout, so records are matched by slice_z. The boundary cache
    // lives inside the entry builder.
    let mut entry_builder = EntryBuilder::new(
        traversal,
        top_surfaces,
        curled_lines,
        nearest_seam_plans,
        island_predecessor,
    );
    // FanMover construction mirrors GCode.cpp:3727-3740 (gate:
    // fan_speedup_time != 0 || fan_kickstart > 0).
    let fan_mover_handle = fan_setup::gate(traversal);

    // `collect_layers_to_print(Print)` (`GCode.cpp:1835-1870`): every print
    // The merged layer-chunk schedule (`layers/schedule.rs`): every print
    // object's layers merged by print z, each chunk's objects in the
    // chained instance order (`collect_layers_to_print` /
    // `chain_print_object_instances`, applied per chunk through
    // `sort_print_object_instances`, `GCode.cpp:4118-4167`). Each print
    // object extrudes around its own build-item placement
    // (`bbs_3mf.cpp:3554-3560`, `GCode.cpp:5380/5403/5437`
    // `set_origin(unscale(instance.shift))`).
    // `GCode.cpp:3284`: the sequential by-object pass, only without a
    // wipe tower.
    let by_object = traversal.resolved.views.full.process.print.print_sequence
        == crate::ProcessPrintSequence::ByObject
        && schedule.tower.is_none();
    let Schedule {
        merged,
        mut raft,
        print_position,
        instances,
        label_inventory,
        labels,
        object_layer_counts,
        mut skirt_schedule,
        tool_ordering,
        layer_count,
        ..
    } = schedule;
    let filament_instances = (state.tags.is_bbl()
        && traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .support_object_skip_flush
            .0
        && traversal.resolved.views.full.process.print.print_sequence
            == crate::ProcessPrintSequence::ByLayer)
        .then_some(&label_inventory);
    // `ToolOrdering::cal_most_used_extruder` (`ToolOrdering.cpp:955-981`) —
    // consumed by the toolchange-boundary timelapse insert gate
    // (`GCode.cpp:5292-5297`).
    state.most_used_extruder = tool_ordering.most_used_extruder(traversal);
    // `GCode.cpp:3284-3297`: by object, every object drives the layer
    // machinery with its own `ToolOrdering(object, final_extruder_id)`
    // (`ToolOrdering.cpp:396-436`), seeded with the previous object's
    // final extruder (`final_extruder_id` starts unset, `GCode.cpp:2721`)
    // and skipped when its ordering yields no first extruder
    // (`GCode.cpp:3293-3296`). The chunks keep their z order; the
    // preamble still fires once on the first emitted chunk.
    struct DrivePhase<'a> {
        object: usize,
        copy: usize,
        copy_filter: Option<usize>,
        merged: &'a [(f64, schedule::LayerToPrint)],
        tool_ordering: &'a super::tool_ordering::ToolOrdering,
    }
    /// The incoming instance's origin in plate millimeters — the upstream
    /// travels to `(0, 0)` after `set_origin(shift)`
    /// (`GCode.cpp:3303/:3327`).
    fn instance_origin(instances: &[Instance], factor: f64) -> [f64; 2] {
        instances
            .first()
            .map(|instance| {
                [
                    instance.shift.x() as f64 * factor,
                    instance.shift.y() as f64 * factor,
                ]
            })
            .unwrap_or([0.0; 2])
    }
    let mut scoped_chunks: Vec<Vec<(f64, schedule::LayerToPrint)>> = Vec::new();
    let mut scoped_orderings: Vec<super::tool_ordering::ToolOrdering> = Vec::new();
    let mut phase_meta: Vec<(usize, usize, usize)> = Vec::new();
    if by_object {
        let mut final_extruder_id = None;
        for (object, object_instances) in instances.iter().enumerate() {
            let ordering = super::tool_ordering::build(
                traversal,
                objects,
                support_layers,
                raft.as_ref(),
                super::tool_ordering::OrderingScope::Object {
                    object,
                    seed_extruder: final_extruder_id,
                },
            )?;
            if ordering.first_extruder.is_none() {
                continue;
            }
            final_extruder_id = ordering.last_extruder();
            // The ordering is built once per object: a second instance of
            // the same object does not rebuild it (`GCode.cpp:3286`), it
            // reruns the same layer machinery after the :3304 transition.
            let ordering_index = scoped_orderings.len();
            scoped_orderings.push(ordering);
            let object_chunks: Vec<(f64, schedule::LayerToPrint)> = merged
                .iter()
                .filter(|(_, entry)| entry.object_index == object)
                .copied()
                .collect();
            for copy in 0..object_instances.len() {
                phase_meta.push((object, copy, ordering_index));
                scoped_chunks.push(object_chunks.clone());
            }
        }
    }
    let phases: Vec<DrivePhase<'_>> = if by_object {
        scoped_chunks
            .iter()
            .zip(&phase_meta)
            .map(|(merged, (object, copy, ordering))| DrivePhase {
                object: *object,
                copy: *copy,
                copy_filter: Some(*copy),
                merged,
                tool_ordering: &scoped_orderings[*ordering],
            })
            .collect()
    } else {
        vec![DrivePhase {
            object: usize::MAX,
            copy: 0,
            copy_filter: None,
            merged: &merged,
            tool_ordering: &tool_ordering,
        }]
    };
    let mut raft_layers = raft
        .as_mut()
        .map(|raft| std::mem::take(&mut raft.layers))
        .unwrap_or_default();

    let mut phase_index = 0usize;
    let mut group_start = 0;
    let mut first_group = true;
    let mut fan_layers_start = output.len();
    let mut last_object_copy: Option<(usize, usize)> = None;
    let mut previous_layer_z = 0.0;
    let mut chunk_index = 0;
    let mut final_chunk = None;
    include!("layers/drive.rs");
    cooling.finish(output);
    // `GCode.cpp:3440-3441` (ByLayer; the per-object completion
    // `:3359-3361` reaches the same point at the final object): once
    // the second-layer block ran with `Enable`, the disable command
    // closes power-loss recovery after `process_layers` and before the
    // wipe-tower finalize (`GCodeWriter.cpp:505-522`).
    output.extend_from_slice(
        crate::gcode_power_loss_recovery::finish_command(
            state.tags.is_bbl(),
            state.options.gcode_flavor,
            state.options.gcode_comments,
            state.power_loss_recovery,
        )
        .as_bytes(),
    );

    let filtered_end = output.len();
    if let Some((tool_ordering, entry, slices, spacing)) = final_chunk {
        tower::emit(
            output,
            state,
            tower::Event::Finalize { layer_count },
            tower::Input {
                context: super::timelapse::Context {
                    traversal,
                    tool_ordering,
                    metadata,
                    first_layer_bounds,
                    layer: super::timelapse::TimelapseLayer {
                        index: chunk_index - 1,
                        z: previous_layer_z,
                        max_z: max_layer_z,
                    },
                },
                entry: &entry,
                builder: &mut entry_builder,
                support: entry
                    .support_layer
                    .map(|index| &support_layers[entry.object_index][index]),
                chunk_slices: &slices,
                spacing,
            },
        )?;
    }
    Ok((
        max_layer_z,
        fan_mover_handle,
        filtered_end,
        fan_layers_start,
    ))
}
