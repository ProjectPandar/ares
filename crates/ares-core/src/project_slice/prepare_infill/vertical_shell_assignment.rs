mod assign;
mod cleanup;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
};

pub(in crate::project_slice) use types::PreparedPostVerticalShellAssignment;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    SolidIntersection,
    InternalDifference,
    InternalVoidDifference,
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellFiltering,
) -> Result<PreparedPostVerticalShellAssignment, SliceError> {
    stage::prepare(prepared)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellAssignment) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell fill-surface assignment geometry is outside the supported Clipper range"
            .to_owned(),
    )
}

pub(super) fn record_commit() {}
