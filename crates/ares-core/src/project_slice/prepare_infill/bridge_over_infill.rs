pub(in crate::project_slice) mod anchored_polygon;
pub(in crate::project_slice) mod automatic_bridge_angle;
pub(in crate::project_slice) mod bridge_rewrite_areas;
pub(in crate::project_slice) mod candidate_anchored_bridge;
mod candidate_boundary_polylines;
pub(in crate::project_slice) mod candidate_bridge_angle;
mod candidate_bridge_area;
pub(in crate::project_slice) mod candidate_bridge_commit;
pub(in crate::project_slice) mod candidate_bridge_postprocessing;
pub(in crate::project_slice) mod candidate_collision_reconstruction;
mod candidate_ordering;
mod candidates;
mod current_layer_context;
pub(in crate::project_slice) mod deep_sparse_area;
pub(in crate::project_slice) mod internal_bridge_angle;
pub(in crate::project_slice) mod internal_bridge_surfaces;
pub(in crate::project_slice) mod internal_infill_rebuild;
pub(in crate::project_slice) mod internal_solid_recomposition;
pub(in crate::project_slice) mod layer_clustering;
mod lower_cluster_subtraction;
pub(in crate::project_slice) mod region_bridge_ensuring_areas;
pub(in crate::project_slice) mod region_bridge_surface_commit;
pub(in crate::project_slice) mod sparse_anchoring;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod transaction;
mod types;

use crate::{
    SliceError, project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
};

pub(in crate::project_slice) use stage::PreparedPostBridgeCandidates;

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostExternalSurfaces,
) -> Result<PreparedPostBridgeCandidates, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    stage::prepare(predecessor)
}

#[cfg(test)]
pub(in crate::project_slice) fn dispose(prepared: PreparedPostBridgeCandidates) {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    stage::dispose(prepared);
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}
