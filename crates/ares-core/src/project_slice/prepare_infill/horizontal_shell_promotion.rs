mod cleanup;
mod promote;
mod stage;

mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::vertical_shell_assignment::PreparedPostVerticalShellAssignment,
};

pub(in crate::project_slice) use types::PreparedPostHorizontalShellPromotion;

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellAssignment,
) -> Result<PreparedPostHorizontalShellPromotion, SliceError> {
    stage::prepare(prepared)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostHorizontalShellPromotion) {
    record_disposal();
    cleanup::successor(prepared);
}

fn record_commit() {}

fn record_disposal() {}
