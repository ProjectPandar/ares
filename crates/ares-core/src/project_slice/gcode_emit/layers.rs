//! GCode.cpp layer-chunk emission, extracted without changing command ordering.
mod boundary;
mod brim_emit;
mod context;
mod entry;
mod entry_builder;
mod fan_setup;
mod group;
mod layer_end;

pub(super) use context::Context;
mod object_entry;
mod object_order;
mod preamble;
mod raft_entry;
mod raft_schedule;
mod schedule;
mod skirt_emit;
mod timelapse;
mod wrapping;

use entry_builder::EntryBuilder;
use schedule::Schedule;

use super::{
    GenerationMetadata, PreparedPostIslandPrintOrder, SliceError, cooling, fan_mover, footprint,
    layer_gcode, motion, object, spiral_vase,
};
use crate::geometry::ExPolygon;

pub(super) fn append(
    prepared: &mut PreparedPostIslandPrintOrder,
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: Context<'_>,
) -> Result<(f64, Option<fan_mover::FanMover>), SliceError> {
    let Context {
        metadata,
        first_layer_bounds,
        start_position,
        bed_cache,
        extruder_offset,
        brim,
        skirt,
        slicer,
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
    let mut cooling = cooling::CoolingState::from_traversal(traversal);
    let mut spiral = spiral_vase::SpiralVaseFilter::from_traversal(traversal, brim.is_some());
    // `GCode.cpp:4629-4630`: upstream tracks `m_max_layer_z` as the
    // running max of `float(print_z)` — the f32 cast absorbs the f64
    // accumulation drift so `{max_layer_z}`/`max_print_z` placeholders
    // read the float-domain top layer, not a raw f64 height sum.
    let mut max_layer_z = 0.0f64;
    let layer_change_template =
        layer_gcode::LayerChangeTemplate::new(traversal, metadata, first_layer_bounds);
    let timelapse_gate = timelapse::gate(traversal, state);
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
    let schedule = schedule::build(traversal, objects, slicer);
    let Schedule {
        per_object_z,
        merged,
        mut raft,
        print_position,
        labels,
        object_layer_counts,
        last_entry,
    } = schedule;
    let mut raft_layers = raft
        .as_mut()
        .map(|raft| std::mem::take(&mut raft.layers))
        .unwrap_or_default();
    // Raft support paths pass through the same DP simplification as
    // object support paths (`PrintObject::simplify_extrusion_path` /
    // `simplify_support_extrusion_path` over the support layers).
    raft_schedule::simplify_layers(traversal, &mut raft_layers);
    let raft_plans = raft.as_ref().map(|raft| &raft.plans);

    let mut group_start = 0;
    let mut first_group = true;
    let mut last_object_copy: Option<usize> = None;
    while group_start < merged.len() {
        let group_z = merged[group_start].0;
        let mut group_end = group_start;
        while group_end < merged.len() && merged[group_end].0 == group_z {
            group_end += 1;
        }
        let mut entries: Vec<(usize, usize)> = merged[group_start..group_end]
            .iter()
            .map(|&(_, object_index, layer_index)| (object_index, layer_index))
            .collect();
        entries.sort_by_key(|(object_index, _)| print_position[*object_index]);
        max_layer_z = max_layer_z.max(f64::from(group_z as f32));
        group_start = group_end;
        // Chunk-wide inputs for the external motion planner
        // (`get_perimeter_spacing_external` averages the perimeter
        // spacing over every object with slices at this print z,
        // `AvoidCrossingPerimeters.cpp:511-529`).
        let chunk_slices: Vec<ExPolygon> = entries
            .iter()
            .flat_map(|&(object_index, layer_index)| {
                traversal.objects[object_index]
                    .slices(layer_index)
                    .unwrap_or(&[])
                    .to_vec()
            })
            .collect();
        let chunk_perimeter_spacing = if entries.is_empty() {
            0.0
        } else {
            entries
                .iter()
                .map(|&(object_index, layer_index)| {
                    f64::from(
                        traversal.objects[object_index]
                            .perimeter_spacing(layer_index)
                            .unwrap_or_default(),
                    )
                })
                .sum::<f64>()
                / entries.len() as f64
        };
        let (first_object, first_layer) = entries[0];
        // The preamble fires on the first PHYSICAL layer chunk — raft layer
        // 0 included (`first_layer` is a sentinel there, so the check keys
        // off `first_group`, not `first_layer == 0`). Upstream emits the
        // filament-start g-code at the raft base, not the object base.
        if first_group {
            preamble::emit(
                output,
                state,
                traversal,
                metadata,
                start_position.as_ref(),
                first_layer_bounds,
            )?;
            first_group = false;
        }
        // Raft chunks carry a `RAFT_SENTINEL_BASE + raft_index` layer index;
        // the cooling fan ramp counts raft layers as the literal first
        // physical layers (raft base = layer 0), so map the sentinel back
        // to the raft index, and shift object layers by the raft count
        // (object layer 0 follows the last raft layer).
        let cooling_layer_index = match raft_schedule::classify(first_layer).ok().flatten() {
            Some(raft_index) => raft_index,
            None => {
                first_layer
                    + raft
                        .as_ref()
                        .and_then(|raft| raft.counts.get(first_object))
                        .copied()
                        .unwrap_or(0)
            }
        };
        // The cooling rewrite window opens after the preamble (the start
        // g-code is not part of the layer's buffered extrusions).
        let layer_output_start = output.len();
        cooling.begin_layer(output, cooling_layer_index);
        state.part_fan_speed = cooling.provisional_part_speed();
        // The change-layer block uses the merged chunk's leading entry
        // (`change_layer` runs once per layer chunk, `GCode.cpp:5685`).
        let (previous_layer_z, boundary_layer_index, object_layer_over_raft, leading_raft) =
            group::chunk_bookkeeping(
                raft.as_ref(),
                raft_plans,
                &per_object_z,
                first_object,
                first_layer,
            );
        let layer_z = group_z;
        // The HEIGHT header keeps the f32-difference quirk (upstream
        // `;HEIGHT:0.200001` artifacts) while layer_z itself stays the
        // f64 print_z for template comparisons.
        let layer_height = f64::from(layer_z as f32) - f64::from(previous_layer_z as f32);
        // The skirt flow height differences the RAW f64 print_z pair and
        // casts to f32 only at the end (`GCode.cpp:4417` —
        // `with_height(float(skirt_done.back() - skirt_done[size-2]))`);
        // pre-casting each z to f32 shifts the layer-3 skirt `mm3_per_mm`
        // one f32 ulp and flips the wipe F2930.735/736 boundary.
        let skirt_height = f64::from((layer_z - previous_layer_z) as f32);
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
            boundary_layer_index,
            object_layer_over_raft,
            layer_z,
            layer_height,
            &mut second_layer_done,
            bed_cache,
        )?;
        let timelapse_context = boundary.timelapse_context;
        // The skirt prints once per layer before any object content
        // (`GCode.cpp:4388+`), on the layers it covers. Raft chunks emit
        // the skirt around the raft polygons (the skirt layer index is
        // the raft layer index — with a raft, the covered layers start
        // at the raft flange).
        skirt_emit::emit(
            output,
            state,
            &entries,
            skirt,
            &raft,
            leading_raft,
            skirt_height,
            &mut entry_builder,
            traversal,
            &chunk_slices,
            chunk_perimeter_spacing,
        );
        // `GCode.cpp:5281-5284, 5302-5305`: the wrapping-detection retract +
        // template insert fire once per layer inside the per-extruder loop
        // — AFTER the combined skirt (`GCode.cpp:5244-5252`) but BEFORE the
        // toolchange and every instance block (object skirt group, brim,
        // `; printing object` marker, content).
        wrapping::emit(output, state, &timelapse_context)?;
        for (entry_position, &(object_index, layer_index)) in entries.iter().enumerate() {
            let is_group_end = entry_position + 1 == entries.len();
            let raft_index = raft_schedule::classify(layer_index).ok().flatten();
            let (source_object_index, _) = traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity();
            let (center_x, center_y) = footprint::object_origin(
                &traversal.project.objects()[source_object_index],
                traversal.scale,
            );
            state.origin = (center_x, center_y);
            state.offset = (center_x - extruder_offset.0, center_y - extruder_offset.1);
            // Raft chunks emit the materialized raft layer; the motion
            // machinery (travel/Z/TYPE/wipe/retract) is identical to the
            // object path.
            if let Some(raft_index) = raft_index {
                raft_entry::emit(
                    output,
                    state,
                    &mut raft_layers,
                    &raft,
                    traversal,
                    object_index,
                    raft_index,
                    &labels,
                    emit_labels,
                    traditional_interlude,
                    timelapse_context,
                    is_group_end,
                )?;
                continue;
            }
            // Upstream re-initializes the avoid-crossing boundaries for
            // every instance's layer (`init_layer(*m_layer)` per
            // `instance_to_print`, `GCode.cpp:5343-5345`); drop the cached
            // boundary so each entry rebuilds it from its own slices.
            state.avoid_boundary = None;
            // `GCode.cpp:5380-5384`: when a new object copy starts, the
            // first travel uses the external motion planner
            // (`use_external_mp_once`).
            if last_object_copy != Some(object_index) {
                state.use_external_mp_once = true;
                last_object_copy = Some(object_index);
                // The `_retract` Hilbert suppression reads the
                // DESTINATION object's region config (`GCode.cpp:7670`,
                // `config()` is the current PrintRegionConfig) — refresh
                // on every object switch so one object's Hilbert pattern
                // does not leak into the next.
                // Upstream `_retract` reads `config()` — the current
                // PrintObject's config; refresh the flags per object.
                // `resolved.objects` is compacted when objects lack
                // printable instances — match by source index
                // (`traversal.rs:27-31`), not by position.
                let region = traversal
                    .resolved
                    .objects
                    .iter()
                    .find(|object| object.source_object_index == source_object_index)
                    .and_then(|object| object.layer_candidates.first())
                    .and_then(|layer| layer.model_parts.first())
                    .map(|part| &part.region);
                if let Some(region) = region {
                    state.options.bottom_surface_pattern_hilbert =
                        region.bottom_surface_pattern == crate::ProcessInfillPattern::HilbertCurve;
                    state.options.top_surface_pattern_hilbert =
                        region.top_surface_pattern == crate::ProcessInfillPattern::HilbertCurve;
                }
            }
            brim_emit::emit(
                output,
                state,
                brim,
                object_index,
                layer_index,
                &mut entry_builder,
                traversal,
                &chunk_slices,
                chunk_perimeter_spacing,
            );
            let object = &mut objects[object_index];
            let Some(layer) = object.get_mut(layer_index) else {
                continue;
            };
            object_entry::emit(
                output,
                state,
                layer,
                &mut spiral,
                &mut entry_builder,
                traversal,
                object_index,
                layer_index,
                &chunk_slices,
                chunk_perimeter_spacing,
                &labels,
                emit_labels,
                traditional_interlude,
                timelapse_gate.traditional_timelapse,
                timelapse_context,
                layer_z,
                layer_height,
                &object_layer_counts,
                last_entry,
                is_group_end,
                layer_output_start,
            )?;
        }
        cooling.finish_layer(output, layer_output_start);
    }
    cooling.finish(output);
    Ok((max_layer_z, fan_mover_handle))
}
