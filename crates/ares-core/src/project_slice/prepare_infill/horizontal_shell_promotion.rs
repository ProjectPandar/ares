mod cleanup;
mod promote;
mod stage;
#[cfg(test)]
mod tests;
mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::vertical_shell_assignment::PreparedPostVerticalShellAssignment,
};

pub(in crate::project_slice) use types::PreparedPostHorizontalShellPromotion;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PromotionEvent {
    RawScheduleVisit,
    NonemptySchedule,
    ParserInvocation,
    MatcherInvocation,
    PromotedSurface,
}

#[cfg(test)]
thread_local! {
    static EVENTS: std::cell::RefCell<Vec<PromotionEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static COMMITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostVerticalShellAssignment,
) -> Result<PreparedPostHorizontalShellPromotion, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    stage::prepare(prepared)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostHorizontalShellPromotion) {
    record_disposal();
    cleanup::successor(prepared);
}

#[cfg(test)]
fn record_event(event: PromotionEvent) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

fn record_commit() {
    #[cfg(test)]
    COMMITS.with(|count| count.set(count.get() + 1));
}

fn record_disposal() {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    INVOCATIONS.with(|count| count.set(0));
    COMMITS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn events() -> Vec<PromotionEvent> {
    EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn commits() -> usize {
    COMMITS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}
