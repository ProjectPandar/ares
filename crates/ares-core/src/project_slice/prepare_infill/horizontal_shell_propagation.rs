mod cleanup;
mod gather;
mod geometry;
mod hooks;
mod propagate;
mod rebuild;
mod stage;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transaction_snapshot;
mod types;
mod window;

use crate::{
    SliceError,
    project_slice::prepare_infill::horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
};

#[cfg(test)]
use hooks::record_gather;
#[cfg(test)]
pub(in crate::project_slice) use hooks::{
    GatherObservation, commits, disposals, events, fail_geometry_at, fail_geometry_at_occurrence,
    gather_observations, geometry_events, invocations, reset_hooks, rollback_snapshots,
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
