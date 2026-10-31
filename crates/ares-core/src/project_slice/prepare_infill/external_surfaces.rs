mod detect_bridge_directions;
mod expand_bridges_detect_orientations;
mod expand_expolygons;
mod expand_merge;
mod group_bridges;
pub(in crate::project_slice) mod merge_bridges;
mod parameters;
mod process;
mod stage;

mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
};

pub(in crate::project_slice) use stage::PreparedPostExternalSurfaces;
pub(in crate::project_slice) use types::{Bridge, ExpansionResult, ExpansionZone};

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostHorizontalShellPropagation,
) -> Result<PreparedPostExternalSurfaces, SliceError> {
    stage::prepare(predecessor)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostExternalSurfaces) {
    stage::dispose(prepared);
}
