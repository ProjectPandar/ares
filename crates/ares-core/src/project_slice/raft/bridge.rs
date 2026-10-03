//! Pipeline bridge: derives the full raft construction inputs from
//! `SliceOptions` and the first object layer contours, using the
//! `Slicing.cpp` formulas already ported in `parameters.rs` (raft
//! split/heights/gap via a minimal `SlicingParameters`), then builds
//! the layer plans (`emit.rs`) ready for `apply_raft_to_stream`.

use super::emit::RaftLayerPlan;
use crate::SliceError;
use crate::geometry::ClipperError;

pub(crate) struct RaftStream {
    pub(crate) plans: Vec<RaftLayerPlan>,
    #[expect(dead_code, reason = "consumed by the skirt wiring slice")]
    pub(crate) support_flow_spacing_mm: f64,
    #[expect(dead_code, reason = "consumed by the skirt wiring slice")]
    pub(crate) first_layer_flow_spacing_mm: f64,
}

pub(super) fn raft_geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("raft geometry error: {error:?}"))
}

mod project;
pub(in crate::project_slice) use project::build_project_raft;
