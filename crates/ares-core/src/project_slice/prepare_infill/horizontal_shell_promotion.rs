mod promote;
mod stage;

mod types;

use crate::{
    CancelToken, SliceError,
    project_slice::prepare_infill::vertical_shell_assignment::PreparedPostVerticalShellAssignment,
};

pub(in crate::project_slice) use types::PreparedPostHorizontalShellPromotion;

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellAssignment,
    cancel: &CancelToken,
) -> Result<PreparedPostHorizontalShellPromotion, SliceError> {
    stage::prepare(prepared, cancel)
}
