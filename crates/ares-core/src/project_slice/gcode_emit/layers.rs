//! GCode.cpp layer-chunk emission, extracted without changing command ordering.
mod boundary;
mod brim_emit;
mod context;
mod cooling_result;
mod entry;
mod entry_builder;
mod fan_setup;
mod group;
mod layer_end;

pub(super) use context::Context;
mod instances;
pub(super) use instances::Instance;
mod object_entry;
mod object_order;
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
) -> Result<(f64, Option<fan_mover::FanMover>, usize), SliceError> {
    let Context {
        metadata,
        first_layer_bounds,
        bed_cache,
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
    let mut cooling = cooling::CoolingState::from_traversal(traversal, state);
    let mut spiral = spiral_vase::SpiralVaseFilter::from_traversal(traversal, brim.is_some());
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
        labels,
        object_layer_counts,
        last_entry,
        mut skirt_schedule,
        tool_ordering,
        layer_count,
        ..
    } = schedule;
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
        for object in 0..objects.len() {
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
            for copy in 0..instances[object].len() {
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
    // `Skirt::make_skirt_loops_per_extruder_*`'s `skirt_done` (`GCode.cpp:
    // 4286-4315`): the print z values the skirt already printed at, shared
    // across the sequential by-object instances.
    let mut skirt_done: Vec<f64> = Vec::new();
    let mut group_start = 0;
    let mut first_group = true;
    let mut last_object_copy: Option<(usize, usize)> = None;
    let mut previous_layer_z = 0.0;
    let mut chunk_index = 0;
    let mut final_chunk = None;
    while phase_index < phases.len() {
        let merged: &[(f64, schedule::LayerToPrint)] = phases[phase_index].merged;
        let phase = &phases[phase_index];
        let tool_ordering: &super::tool_ordering::ToolOrdering = phases[phase_index].tool_ordering;
        if group_start >= merged.len() {
            phase_index += 1;
            group_start = 0;
            if let Some(phase) = phases.get(phase_index) {
                // `GCode.cpp:3309-3342`: the instance transition between
                // the sequential objects; `:3369-3372` resets the
                // second-layer flag for the incoming instance, and the
                // layer height restarts from the object's own first
                // layer (the per-object `process_layers` z tracking).
                let origin = instance_origin(
                    &instances[phase.object][phase.copy.min(
                        instances[phase.object].len().saturating_sub(1),
                    )..],
                    traversal.scale.factor(),
                );
                let _primed = transition::emit(
                    output,
                    state,
                    transition::Transition {
                        traversal,
                        metadata,
                        first_layer_bounds,
                        finished_objects: phase_index,
                        initial_extruder: phase.tool_ordering.first_extruder.unwrap_or(0),
                        printing_extruders: phase.tool_ordering.all_extruders().to_vec(),
                        lift_z: max_layer_z + state.options.z_hop,
                        origin,
                        bed_cache,
                    },
                )?;
                let _ = _primed;
                previous_layer_z = 0.0;
                second_layer_done = false;
            }
            continue;
        }
        let group_z = merged[group_start].0;
        let mut group_end = group_start;
        while group_end < merged.len() && merged[group_end].0 == group_z {
            group_end += 1;
        }
        let mut entries: Vec<schedule::LayerToPrint> = merged[group_start..group_end]
            .iter()
            .map(|&(_, entry)| entry)
            .collect();
        let (cooling_layer_index, cooling_buffer_flush) = cooling_result::metadata(
            &entries,
            traversal,
            support_layers,
            group_end == merged.len(),
        );
        group_start = group_end;
        if tool_ordering.tools_for_layer(group_z).is_empty() {
            cooling.empty_layer(output, cooling_layer_index, group_end == merged.len());
            continue;
        }
        entries.sort_by_key(|entry| print_position[entry.object_index]);
        max_layer_z = max_layer_z.max(f64::from(group_z as f32));
        // Chunk-wide inputs for the external motion planner
        // (`get_perimeter_spacing_external` averages the perimeter
        // spacing over every object with slices at this print z,
        // `AvoidCrossingPerimeters.cpp:511-529`).
        let (chunk_slices, chunk_perimeter_spacing) = group::geometry(traversal, &entries);
        // The preamble fires on the first PHYSICAL layer chunk — raft layer
        // 0 included (`first_layer` is a sentinel there, so the check keys
        // off `first_group`, not `first_layer == 0`). Upstream emits the
        // filament-start g-code at the raft base, not the object base.
        if first_group {
            preamble::emit(output, state, traversal, metadata, first_layer_bounds)?;
            tower::emit(
                output,
                state,
                tower::Event::Initialize,
                tower::Input {
                    context: super::timelapse::Context {
                        traversal,
                        metadata,
                        first_layer_bounds,
                        layer: super::timelapse::TimelapseLayer {
                            index: 0,
                            z: 0.0,
                            max_z: 0.0,
                        },
                    },
                    entry: &entries[0],
                    builder: &mut entry_builder,
                    support: entries[0]
                        .support_layer
                        .map(|index| &support_layers[entries[0].object_index][index]),
                    chunk_slices: &chunk_slices,
                    spacing: chunk_perimeter_spacing,
                },
            )?;
            first_group = false;
        }
        // The cooling rewrite window opens after the preamble (the start
        // g-code is not part of the layer's buffered extrusions).
        let layer_output_start = output.len();
        cooling.begin_layer(output, cooling_layer_index);
        state.part_fan_speed = cooling.provisional_part_speed();
        // The change-layer block uses the merged chunk's leading entry
        // (`change_layer` runs once per layer chunk, `GCode.cpp:5685`).
        let slicing = traversal.objects[entries[0].object_index].slicing_parameters();
        let object_layer_over_raft = entries[0].object_layer == Some(0)
            && slicing.base_raft_layers + slicing.interface_raft_layers > 0;
        let layer_z = group_z;
        // The HEIGHT header keeps the f32-difference quirk (upstream
        // `;HEIGHT:0.200001` artifacts) while layer_z itself stays the
        // f64 print_z for template comparisons.
        let layer_height = f64::from(layer_z as f32) - f64::from(previous_layer_z as f32);
        let boundary = boundary::append(
            output,
            state,
            &mut spiral,
            boundary::Boundary {
                traversal,
                layer_change_template: &layer_change_template,
                metadata,
                first_layer_bounds,
            },
            boundary::LayerBoundary {
                layer_index: chunk_index,
                object_layer_over_raft,
                layer_z,
                layer_height,
                second_layer_done: &mut second_layer_done,
                bed_cache,
            },
        )?;
        let timelapse_context = boundary.timelapse_context;
        state.layer_index = cooling_layer_index;
        state.on_first_layer = group::on_first_layer(traversal, &entries, support_layers, &raft);
        // The skirt prints once per layer before any object content
        // (`GCode.cpp:4388+`), gated by the `skirt_done` vector:
        // `Skirt::make_skirt_loops_per_extruder_1st_layer` emits when the
        // vector is empty and records the print z
        // (`GCode.cpp:4292-4295`); `..._other_layers` keeps emitting while
        // `skirt_done.size() < skirt_height` (`GCode.cpp:4311-4315`). The
        // vector is shared across the sequential by-object instances, so a
        // later object restarting at the same z does not re-emit. Raft
        // chunks emit the skirt around the raft polygons (the skirt layer
        // index is the raft layer index — with a raft, the covered layers
        // start at the raft flange).
        let skirt_height = traversal
            .resolved
            .views
            .full
            .process
            .print
            .skirt_height
            .0
            .max(0) as usize;
        if skirt_done.is_empty() || skirt_done.len() < skirt_height {
            skirt_emit::emit(
                output,
                state,
                skirt_emit::SkirtEmission {
                    entries: &entries,
                    support: support_layers,
                    skirt,
                    raft: &raft,
                    print_z: layer_z,
                    schedule: &mut skirt_schedule,
                    entry_builder: &mut entry_builder,
                    traversal,
                    chunk_slices: &chunk_slices,
                    chunk_perimeter_spacing,
                },
            );
            skirt_done.push(layer_z);
        }
        // `GCode.cpp:5281-5284, 5302-5305`: the wrapping-detection retract +
        // template insert fire once per layer inside the per-extruder loop
        // — AFTER the combined skirt (`GCode.cpp:5244-5252`) but BEFORE the
        // toolchange and every instance block (object skirt group, brim,
        // `; printing object` marker, content).
        let layer_tools = tool_ordering.for_layer(group_z);
        let has_tower = layer_tools.has_wipe_tower && state.wipe_tower.is_some();
        if has_tower {
            state.wipe_tower.as_mut().unwrap().next_layer();
        }
        let wiping = &layer_tools.wiping;
        let mut by_tool = tool_buckets::build(tool_buckets::Input {
            traversal,
            entries: &entries,
            instances: &instances,
            tools: tool_ordering.tools_for_layer(group_z),
            wiping,
            objects,
            support: support_layers,
            raft: &mut raft_layers,
        });
        let last_tool = by_tool.last().map(|(tool, _)| *tool);
        // By object, a phase drives ONE instance: the same-object copies
        // split into consecutive phases with the :3304 transition between
        // them (`GCode.cpp:3279-3304`), so the bucket set shrinks to the
        // phase's copy. By layer keeps every copy.
        if let Some(copy) = phase.copy_filter {
            for (_, tool_entries) in by_tool.iter_mut() {
                tool_entries.retain(|bucket| bucket.instance.copy == copy);
            }
        }
        let mut final_entry = entries[0];
        for (tool, tool_entries) in &mut by_tool {
            wrapping::emit(output, state, &timelapse_context)?;
            let needs_change = !state.extruder_selected || state.extruders.active_id() != *tool;
            if has_tower {
                tower::emit(
                    output,
                    state,
                    tower::Event::Change {
                        tool: *tool,
                        finish_layer: Some(*tool) == last_tool,
                    },
                    tower::Input {
                        context: timelapse_context,
                        entry: &entries[0],
                        builder: &mut entry_builder,
                        support: entries[0]
                            .support_layer
                            .map(|index| &support_layers[entries[0].object_index][index]),
                        chunk_slices: &chunk_slices,
                        spacing: chunk_perimeter_spacing,
                    },
                )?;
                state.last_feature = Some("Prime tower");
            } else if needs_change {
                super::tool_transition::emit(output, state, *tool, timelapse_context)?;
            }
            if needs_change {
                last_object_copy = None;
            }
            let entry_count = tool_entries.len();
            for (pass, entry_position) in [tool_buckets::Pass::Wiping, tool_buckets::Pass::Normal]
                .into_iter()
                .filter(|pass| *pass == tool_buckets::Pass::Normal || wiping.has_overrides())
                .flat_map(|pass| (0..entry_count).map(move |position| (pass, position)))
            {
                final_entry = tool_entries[entry_position].source;
                tool_entry::emit(
                    output,
                    state,
                    &mut tool_entries[entry_position],
                    tool_entry::Input {
                        traversal,
                        pass,
                        purging_finished: pass == tool_buckets::Pass::Normal
                            && entry_position == 0
                            && wiping.has_overrides(),
                        support_layers,
                        raft: &raft,
                        labels: &labels,
                        emit_labels,
                        traditional_interlude,
                        traditional_timelapse: timelapse_gate.traditional_timelapse,
                        timelapse_context,
                        is_group_end: pass == tool_buckets::Pass::Normal
                            && entry_position + 1 == entry_count
                            && Some(*tool) == last_tool,
                        last_object_copy: &mut last_object_copy,
                        entry_builder: &mut entry_builder,
                        chunk_slices: &chunk_slices,
                        chunk_perimeter_spacing,
                        brim,
                        spiral: &mut spiral,
                        layer_z,
                        layer_height,
                        object_layer_counts: &object_layer_counts,
                        last_entry,
                        layer_output_start,
                    },
                )?;
            }
        }
        cooling.finish_layer(output, layer_output_start, cooling_buffer_flush);
        if state.wipe_tower.is_some() {
            final_chunk = Some((final_entry, chunk_slices, chunk_perimeter_spacing));
        }
        previous_layer_z = layer_z;
        chunk_index += 1;
        state.adaptive_pa_layer = chunk_index;
    }
    cooling.finish(output);
    let filtered_end = output.len();
    if let Some((entry, slices, spacing)) = final_chunk {
        tower::emit(
            output,
            state,
            tower::Event::Finalize { layer_count },
            tower::Input {
                context: super::timelapse::Context {
                    traversal,
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
    Ok((max_layer_z, fan_mover_handle, filtered_end))
}
