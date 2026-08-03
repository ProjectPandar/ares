use std::cell::{Cell, RefCell};

use super::geometry::GeometryStep;

mod geometry;
mod preflight;

thread_local! {
    static GUARD_INVOCATIONS: Cell<usize> = const { Cell::new(0) };
    static GEOMETRY_EVENTS: RefCell<Vec<GeometryStep>> = const { RefCell::new(Vec::new()) };
    static GEOMETRY_FAILURE: Cell<Option<GeometryStep>> = const { Cell::new(None) };
}

pub(super) fn observe_guard() {
    GUARD_INVOCATIONS.with(|count| count.set(count.get() + 1));
}

pub(super) fn observe_step(step: GeometryStep) -> bool {
    GEOMETRY_EVENTS.with(|events| events.borrow_mut().push(step));
    GEOMETRY_FAILURE.with(|failure| failure.get() == Some(step))
}

pub(super) fn reset_geometry_hooks() {
    GEOMETRY_EVENTS.with(|events| events.borrow_mut().clear());
    GEOMETRY_FAILURE.with(|failure| failure.set(None));
}

pub(super) fn fail_at(step: GeometryStep) {
    GEOMETRY_FAILURE.with(|failure| failure.set(Some(step)));
}

pub(super) fn geometry_events() -> Vec<GeometryStep> {
    GEOMETRY_EVENTS.with(|events| events.borrow().clone())
}
