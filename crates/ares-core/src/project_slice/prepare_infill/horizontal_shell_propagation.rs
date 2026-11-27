mod gather;
mod geometry;
mod propagate;
mod rebuild;
mod stage;

mod types;
mod window;

use crate::{
    CancelToken, SliceError,
    project_slice::prepare_infill::horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
};

pub(in crate::project_slice) use types::PreparedPostHorizontalShellPropagation;

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostHorizontalShellPromotion,
    cancel: &CancelToken,
) -> Result<PreparedPostHorizontalShellPropagation, SliceError> {
    stage::prepare(prepared, cancel)
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "horizontal-shell propagation geometry is outside the supported Clipper range".to_owned(),
    )
}
