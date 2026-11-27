mod filter;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{
    CancelToken, SliceError, project_slice::prepare_infill::vertical_shell_regularization,
};

pub(in crate::project_slice) use types::PreparedPostVerticalShellFiltering;

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_regularization::PreparedPostVerticalShellRegularization,
    cancel: &CancelToken,
) -> Result<PreparedPostVerticalShellFiltering, SliceError> {
    let filters = stage::filter(&prepared, cancel)?;
    let vertical_shell_regularization::PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        ..
    } = prepared;
    Ok(PreparedPostVerticalShellFiltering {
        predecessor,
        objects,
        filters,
    })
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
