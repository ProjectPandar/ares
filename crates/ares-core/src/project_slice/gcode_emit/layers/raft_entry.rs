//! Raft entry emission inside the per-chunk object loop — raft chunks
//! emit the materialized raft layer; the motion machinery
//! (travel/Z/TYPE/wipe/retract) is identical to the object path.

use super::super::{SliceError, motion, object::ObjectLabels, timelapse as timelapse_core};
use super::entry::EntryGeometry;
use super::raft_schedule::RaftSchedule;
use crate::project_slice::island_print_order::OrderedExtrusionLayer;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[expect(
    clippy::too_many_arguments,
    reason = "the raft branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    raft_layers: &mut [Vec<OrderedExtrusionLayer>],
    raft: &Option<RaftSchedule>,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    raft_index: usize,
    labels: &[Option<ObjectLabels>],
    emit_labels: bool,
    traditional_interlude: bool,
    timelapse_context: timelapse_core::Context<'_>,
    is_group_end: bool,
) -> Result<(), SliceError> {
    let Some(layer) = raft_layers
        .get_mut(object_index)
        .and_then(|layers| layers.get_mut(raft_index))
    else {
        return Ok(());
    };
    let raft_polygons = raft
        .as_ref()
        .and_then(|raft| raft.plans.get(object_index))
        .and_then(|plans| plans.get(raft_index))
        .map(|plan| {
            plan.polygons
                .iter()
                .map(|polygon| crate::geometry::ExPolygon::new(polygon.clone(), Vec::new()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let geometry = EntryGeometry {
        top_surfaces: Vec::new(),
        lower_boundary_lines: Vec::new(),
        nearest_penalties: None,
        layer_slices: raft_polygons.clone().into(),
        internal_surfaces: Vec::new(),
        chunk_slices: raft_polygons,
        chunk_perimeter_spacing: 0.0,
    };
    let motion_geometry = geometry.view_raft(traversal.scale);
    // Raft layers belong to the same print instance and
    // carry its object labels (`GCode.cpp:5348-5352` fires
    // per `instance_to_print`, raft layers included).
    if let Some(labels) = &labels[object_index] {
        labels.queue_start(output, state, emit_labels);
    }
    let timelapse_inserted = motion::emit_layer(
        output,
        layer,
        (motion_geometry, |_| {
            traversal
                .resolved
                .views
                .full
                .process
                .region
                .sparse_infill_density
                .0
        }),
        state,
        |output, state| {
            timelapse_core::append_traditional(
                traditional_interlude,
                output,
                state,
                timelapse_context,
            )
        },
    )?;
    if let Some(labels) = &labels[object_index] {
        labels.queue_stop(
            output,
            state,
            emit_labels,
            timelapse_inserted && is_group_end,
        );
    }
    if is_group_end {
        motion::end_layer_for_timelapse(output, state);
    }
    Ok(())
}
