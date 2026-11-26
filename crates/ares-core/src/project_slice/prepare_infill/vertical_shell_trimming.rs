mod stage;

pub(in crate::project_slice::prepare_infill) mod trim;
pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_projection};

pub(in crate::project_slice) use types::PreparedPostVerticalShellTrim;

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shell_projection::PreparedPostVerticalShellProjection,
) -> Result<PreparedPostVerticalShellTrim, SliceError> {
    let trims = stage::trim(&prepared)?;
    let vertical_shell_projection::PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        ..
    } = prepared;
    Ok(PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        trims,
    })
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell internal trimming geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
