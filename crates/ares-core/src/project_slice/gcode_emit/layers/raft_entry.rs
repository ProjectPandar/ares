//! Raft entry emission inside the per-chunk object loop — raft chunks
//! emit the materialized raft layer; the motion machinery
//! (travel/Z/TYPE/wipe/retract) is identical to the object path.

use super::super::{SliceError, motion, object::ObjectLabels, timelapse as timelapse_core};
use super::entry::EntryGeometry;
use super::raft_schedule::RaftSchedule;
use crate::project_slice::island_print_order::{OrderedExtrusionLayer, visits::InstanceVisit};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[expect(
    clippy::too_many_arguments,
    reason = "the raft branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    (layer, visit): (&mut OrderedExtrusionLayer, &InstanceVisit),
    raft: &Option<RaftSchedule>,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    raft_index: usize,
    labels: Option<&ObjectLabels>,
    emit_labels: bool,
    traditional_interlude: bool,
    timelapse_context: timelapse_core::Context<'_>,
    is_group_end: bool,
) -> Result<(), SliceError> {
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
        previous_curled: &[],
        top_surfaces: Vec::new(),
        lower_boundary_lines: Vec::new(),
        nearest_penalties: None,
        layer_slices: raft_polygons.clone().into(),
        internal_surfaces: Vec::new(),
        chunk_slices: raft_polygons,
        chunk_perimeter_spacing: 0.0,
    };
    let motion_geometry = geometry.view_raft(timelapse_context);
    let plan = &raft
        .as_ref()
        .expect("a materialized raft has its schedule")
        .plans[object_index][raft_index];
    let first_layer = raft_index == 0 && plan.z.bottom_z.abs() < 0.0001;
    state.on_first_layer = first_layer;
    state.active_layer_print_z = plan.z.print_z;
    // Raft layers belong to the same print instance and
    // carry its object labels (`GCode.cpp:5348-5352` fires
    // per `instance_to_print`, raft layers included).
    if let Some(labels) = labels {
        labels.queue_start(output, state, emit_labels);
    }
    let reversal = state.reversal_scope(
        traversal,
        object_index,
        motion::SharedLayerId::Raft(raft_index),
    );
    motion::emit_layer(
        output,
        (layer, visit, None),
        (
            motion::EmissionPhase {
                geometry: motion_geometry,
                reversal,
            },
            first_layer,
            |_, _, current_seam_gap: &mut crate::FloatOrPercent| {
                let region = &traversal.resolved.views.full.process.region;
                *current_seam_gap = region.seam_gap;
                (
                    region.sparse_infill_density.0,
                    region.is_infill_first.0,
                    super::super::small_area::Gate {
                        enabled: region.small_area_infill_flow_compensation.0,
                        bottom: region.bottom_surface_pattern,
                        internal: region.internal_solid_infill_pattern,
                        top: region.top_surface_pattern,
                    },
                )
            },
        ),
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
    if let Some(labels) = labels {
        labels.queue_stop(output, state, emit_labels);
    }
    if is_group_end {
        motion::end_layer_for_timelapse(output, state);
    }
    Ok(())
}
