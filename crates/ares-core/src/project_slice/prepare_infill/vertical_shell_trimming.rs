mod cleanup;
mod stage;

pub(in crate::project_slice::prepare_infill) mod trim;
pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_projection};

pub(in crate::project_slice) use types::PreparedPostVerticalShellTrim;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    SafetyOffset,
    SafetyIntersection,
    Difference,
    EmptyGate,
    SolidAppend,
}

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_projection::PreparedPostVerticalShellProjection,
) -> Result<PreparedPostVerticalShellTrim, SliceError> {
    let trims = match stage::trim(&prepared) {
        Ok(trims) => trims,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shell_projection::PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        caches,
        projections,
    } = prepared;
    Ok(PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        caches,
        projections,
        trims,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellTrim) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell internal trimming geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
