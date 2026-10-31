mod cleanup;
mod gather;
mod geometry;
mod hooks;
mod propagate;
mod rebuild;
mod stage;

mod types;
mod window;

use crate::{
    SliceError,
    project_slice::prepare_infill::horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
};

pub(in crate::project_slice) use hooks::{GeometryStep, PropagationEvent};
use hooks::{geometry_step, range_error, record_commit, record_disposal, record_event};
pub(in crate::project_slice) use types::{PreparedPostHorizontalShellPropagation, SourceKind};

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostHorizontalShellPromotion,
) -> Result<PreparedPostHorizontalShellPropagation, SliceError> {
    hooks::record_invocation();
    stage::prepare(prepared)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostHorizontalShellPropagation) {
    record_disposal();
    cleanup::successor(prepared);
}
