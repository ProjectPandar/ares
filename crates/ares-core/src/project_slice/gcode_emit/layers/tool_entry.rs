//! Per-tool instance dispatch from GCode.cpp:5322–5560.

use super::super::{
    SliceError, brim::BrimPlan, motion, object::ObjectLabels, spiral_vase, timelapse,
};
use super::{
    brim_emit, entry_builder::EntryBuilder, object_entry, raft_entry, support_entry, tool_buckets,
};
use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) struct Input<'a, 'b> {
    pub(super) visit: &'a crate::project_slice::island_print_order::visits::InstanceVisit,
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) pass: tool_buckets::Pass,
    pub(super) purging_finished: bool,
    pub(super) support_layers: &'a mut [Vec<InstalledSupportLayer>],
    pub(super) raft: &'a Option<super::RaftSchedule>,
    pub(super) labels: &'a [Vec<Option<ObjectLabels>>],
    pub(super) emit_labels: bool,
    pub(super) traditional_interlude: bool,
    pub(super) traditional_timelapse: bool,
    pub(super) timelapse_context: timelapse::Context<'a>,
    pub(super) is_group_end: bool,
    pub(super) last_object_copy: &'a mut Option<(usize, usize)>,
    pub(super) entry_builder: &'a mut EntryBuilder<'b>,
    pub(super) chunk_slices: &'a [ExPolygon],
    pub(super) chunk_perimeter_spacing: f64,
    pub(super) brim: &'a [Option<BrimPlan>],
    pub(super) skirt: &'a Option<super::super::skirt::SkirtPlan>,
    pub(super) object_skirt_done: &'a mut [(usize, f64)],
    pub(super) spiral: &'a mut spiral_vase::SpiralVaseFilter,
    pub(super) layer_z: f64,
    pub(super) layer_height: f64,
    pub(super) object_layer_counts: &'a [usize],
    pub(super) layer_output_start: usize,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    bucket: &mut tool_buckets::Entry,
    input: Input<'_, '_>,
) -> Result<(), SliceError> {
    let Input {
        visit,
        traversal,
        pass,
        purging_finished,
        support_layers,
        raft,
        labels,
        emit_labels,
        traditional_interlude,
        traditional_timelapse,
        timelapse_context,
        is_group_end,
        last_object_copy,
        entry_builder,
        chunk_slices,
        chunk_perimeter_spacing,
        brim,
        skirt,
        object_skirt_done,
        spiral,
        layer_z,
        layer_height,
        object_layer_counts,
        layer_output_start,
    } = input;
    if purging_finished {
        output.extend_from_slice(b"; PURGING FINISHED\n");
    }
    let body = match pass {
        tool_buckets::Pass::Wiping => &mut bucket.wipe_body,
        tool_buckets::Pass::Normal => &mut bucket.body,
    };
    let support_role = bucket.support_role.filter(|_| {
        visit.first_visit && bucket.support_wiping == (pass == tool_buckets::Pass::Wiping)
    });
    let entry = &bucket.source;
    let object_index = entry.object_index;
    let labels = labels[object_index][bucket.instance.copy].as_ref();
    let raft_index = entry.raft_layer;
    let (source_object_index, _) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    // `GCode.cpp:5336-5338`: `generate_object_skirt_group` runs inside
    // the FIRST `print_wipe_extrusions` loop only (the wiping loop when
    // extrusions are overridden, the single normal loop otherwise),
    // before the instance's labels (`:5349`); the `object_skirt_done`
    // tracking keeps later loops from re-emitting.
    if visit.first_visit
        && (entry.object_layer.is_some()
            || entry.support_layer.is_some()
            || entry.raft_layer.is_some())
    {
        super::object_skirt::emit(
            output,
            state,
            super::object_skirt::Input {
                skirt,
                done: object_skirt_done,
                object: object_index,
                copy: bucket.instance.copy,
                layer: entry.object_layer,
                builder: entry_builder,
                traversal,
                slices: chunk_slices,
                spacing: chunk_perimeter_spacing,
                support_layers,
                raft,
                support_layer: entry.support_layer,
                raft_layer: entry.raft_layer,
                print_z: layer_z,
            },
        )?;
    }
    let shift = bucket.instance.shift;
    // `set_origin(unscale(offset))` (`GCode.cpp:7643`): the global instance shift; the writer
    // subtracts the plate origin.
    motion::set_origin(
        state,
        (
            traversal.scale.unscale(shift.x()),
            traversal.scale.unscale(shift.y()),
        ),
    );
    // Raft chunks emit the materialized raft layer; the motion
    // machinery (travel/Z/TYPE/wipe/retract) is identical to the
    // object path.
    if let Some(raft_index) = raft_index {
        let Some(body) = body.as_mut() else {
            return Ok(());
        };
        state.current_seam_gap = traversal.resolved.views.full.process.region.seam_gap;
        state.apply_object_config(&traversal.print_object_config(object_index).0.object);
        raft_entry::emit(
            output,
            state,
            (body, visit),
            raft,
            traversal,
            object_index,
            raft_index,
            labels,
            emit_labels,
            traditional_interlude,
            timelapse_context,
            is_group_end,
        )?;
        return Ok(());
    }
    // Upstream re-initializes the avoid-crossing boundaries for
    // every instance's layer (`init_layer(*m_layer)` per
    // `instance_to_print`, `GCode.cpp:5343-5345`); drop the cached
    // boundary so each entry rebuilds it from its own slices.
    state.avoid_boundary = None;
    // `GCode.cpp:5380-5384`: when a new object copy starts, the
    // first travel uses the external motion planner
    // (`use_external_mp_once`).
    let object_copy = (object_index, bucket.instance.copy);
    if *last_object_copy != Some(object_copy) {
        state.use_external_mp_once = true;
        *last_object_copy = Some(object_copy);
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
    // GCode.cpp:5335 emits the object brim ahead of the same instance's
    // support extrusions (:5419-5426); the layer state it reads is the
    // object-layer state assigned at :5343-5344, so hoist those
    // assignments ahead of the brim instead of leaving the previous
    // entry's values in place.
    let object_layer_state = entry.object_layer.inspect(|&layer_index| {
        let physical_layer = &traversal.objects[object_index].physical_layers()[layer_index];
        state.layer_index = physical_layer.id;
        state.active_layer_print_z = physical_layer.print_z;
        state.on_first_layer =
            physical_layer.id == 0 && (physical_layer.print_z - physical_layer.height).abs() < 1e-4;
        let slicing = traversal.objects[object_index].slicing_parameters();
        state.object_layer_over_raft =
            layer_index == 0 && slicing.base_raft_layers + slicing.interface_raft_layers > 0;
    });
    if let Some(layer_index) = object_layer_state
        && visit.first_visit
    {
        brim_emit::emit(
            output,
            state,
            (brim, skirt),
            object_index,
            bucket.instance.copy,
            layer_index,
            entry_builder,
            traversal,
            chunk_perimeter_spacing,
        )?;
    }
    // GCode.cpp:5334-5342 resets region config only AFTER skirt and brim.
    state.current_seam_gap = traversal.resolved.views.full.process.region.seam_gap;
    state.apply_object_config(&traversal.print_object_config(object_index).0.object);
    if let Some(selected_role) = support_role {
        let support_index = entry.support_layer.expect("support bucket retains layer");
        // GCode.cpp:5388 runs the support extrusions with
        // `m_object_layer_over_raft = false` and :5429-5430 restores the
        // object-layer value before the islands.
        let saved_over_raft = state.object_layer_over_raft;
        state.object_layer_over_raft = false;
        support_entry::emit(
            output,
            state,
            &mut support_layers[object_index][support_index],
            selected_role,
            traversal,
            object_index,
            entry.object_layer,
            entry_builder,
            chunk_slices,
            chunk_perimeter_spacing,
            labels,
            emit_labels,
            body.is_none(),
            is_group_end,
            traditional_timelapse,
            timelapse_context,
        )?;
        state.object_layer_over_raft = saved_over_raft;
    }
    let Some(layer_index) = object_layer_state else {
        return Ok(());
    };
    let Some(layer) = body.as_mut() else {
        // GCode.cpp:5348–5372,5478–5494 labels every selected instance,
        // including an empty half of the wiping/normal two-pass traversal.
        if support_role.is_none()
            && let Some(labels) = labels
        {
            labels.queue_start(output, state, emit_labels);
            labels.queue_stop(output, state, emit_labels);
        }
        return Ok(());
    };
    object_entry::emit(
        output,
        state,
        (layer, visit),
        spiral,
        entry_builder,
        traversal,
        object_index,
        layer_index,
        support_role.is_some(),
        chunk_perimeter_spacing,
        labels,
        emit_labels,
        traditional_interlude,
        traditional_timelapse,
        timelapse_context,
        layer_z,
        layer_height,
        object_layer_counts,
        is_group_end,
        layer_output_start,
    )?;
    Ok(())
}
