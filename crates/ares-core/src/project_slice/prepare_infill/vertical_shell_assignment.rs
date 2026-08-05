mod assign;
mod cleanup;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
};

pub(in crate::project_slice) use types::PreparedPostVerticalShellAssignment;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    SolidIntersection,
    InternalDifference,
    InternalVoidDifference,
}

#[cfg(test)]
thread_local! {
    static EVENTS: std::cell::RefCell<Vec<GeometryStep>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static FAILURE: std::cell::Cell<Option<(GeometryStep, usize)>> =
        const { std::cell::Cell::new(None) };
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static COMMITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellFiltering,
) -> Result<PreparedPostVerticalShellAssignment, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    stage::prepare(prepared)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellAssignment) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(test)]
    {
        EVENTS.with(|events| events.borrow_mut().push(_step));
        let failed = FAILURE.with(|failure| match failure.get() {
            Some((failed_step, remaining)) if failed_step == _step && remaining == 1 => true,
            Some((failed_step, remaining)) if failed_step == _step => {
                failure.set(Some((failed_step, remaining - 1)));
                false
            }
            _ => false,
        });
        if failed {
            return Err(range_error());
        }
    }
    Ok(())
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell fill-surface assignment geometry is outside the supported Clipper range"
            .to_owned(),
    )
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_geometry_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    FAILURE.with(|failure| failure.set(None));
    COMMITS.with(|commits| commits.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    fail_geometry_at_occurrence(step, 1);
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at_occurrence(step: GeometryStep, occurrence: usize) {
    assert!(occurrence > 0);
    FAILURE.with(|failure| failure.set(Some((step, occurrence))));
}

pub(super) fn record_commit() {
    #[cfg(test)]
    COMMITS.with(|commits| commits.set(commits.get() + 1));
}

#[cfg(test)]
pub(in crate::project_slice) fn commits() -> usize {
    COMMITS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn geometry_events() -> Vec<GeometryStep> {
    EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}
