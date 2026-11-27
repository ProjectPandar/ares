mod assign;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{
    CancelToken, SliceError,
    project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
};

pub(in crate::project_slice) use types::PreparedPostVerticalShellAssignment;

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellFiltering,
    cancel: &CancelToken,
) -> Result<PreparedPostVerticalShellAssignment, SliceError> {
    stage::prepare(prepared, cancel)
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell fill-surface assignment geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
