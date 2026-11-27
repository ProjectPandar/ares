mod regularize;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{CancelToken, SliceError, project_slice::prepare_infill::vertical_shell_trimming};

pub(in crate::project_slice) use types::PreparedPostVerticalShellRegularization;

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_trimming::PreparedPostVerticalShellTrim,
    cancel: &CancelToken,
) -> Result<PreparedPostVerticalShellRegularization, SliceError> {
    let regularizations = stage::regularize(&prepared, cancel)?;
    let vertical_shell_trimming::PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        trims,
    } = prepared;
    Ok(PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        trims,
        regularizations,
    })
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell regularization geometry is outside the supported Clipper range".to_owned(),
    )
}

pub(in crate::project_slice::prepare_infill) fn minimum_spacing(solid_infill_spacing: i64) -> f32 {
    regularize::min_perimeter_infill_spacing(solid_infill_spacing)
}
