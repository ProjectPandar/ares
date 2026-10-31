mod cleanup;
mod filter;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_regularization};

pub(in crate::project_slice) use types::PreparedPostVerticalShellFiltering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    NeighborIntersection,
    ClosingGrow,
    ClosingShrink,
    CandidateScan,
    VisibilityDifference,
    CandidateExpansion,
    ProtectionDifference,
    EmptyGate,
}

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> Result<PreparedPostVerticalShellFiltering, SliceError> {
    let filters = match stage::filter(&prepared) {
        Ok(filters) => filters,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shell_regularization::PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    } = prepared;
    Ok(PreparedPostVerticalShellFiltering {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellFiltering) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
