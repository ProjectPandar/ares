mod cleanup;
mod regularize;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_trimming};

pub(in crate::project_slice) use types::PreparedPostVerticalShellRegularization;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    Union,
    Offset2First,
    Offset2Second,
    Shrink,
}

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_trimming::PreparedPostVerticalShellTrim,
) -> Result<PreparedPostVerticalShellRegularization, SliceError> {
    let regularizations = match stage::regularize(&prepared) {
        Ok(regularizations) => regularizations,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shell_trimming::PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        caches,
        projections,
        trims,
    } = prepared;
    Ok(PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellRegularization) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell regularization geometry is outside the supported Clipper range".to_owned(),
    )
}

pub(in crate::project_slice::prepare_infill) fn minimum_spacing(solid_infill_spacing: i64) -> f32 {
    regularize::min_perimeter_infill_spacing(solid_infill_spacing)
}
