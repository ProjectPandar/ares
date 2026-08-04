mod cleanup;
mod gather;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shells};

pub(in crate::project_slice) use types::PreparedPostVerticalShellProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    TopVisit,
    BottomVisit,
    HoleIntersection,
    ShellUnion,
    TopAnchorOffset,
    TopAnchorIntersection,
    BottomAnchorOffset,
    BottomAnchorIntersection,
}

#[cfg(test)]
thread_local! {
    static EVENTS: std::cell::RefCell<Vec<GeometryStep>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static FAILURE: std::cell::Cell<Option<(GeometryStep, usize)>> =
        const { std::cell::Cell::new(None) };
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shells::PreparedPostVerticalShellCache,
) -> Result<PreparedPostVerticalShellProjection, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    let projections = match stage::project(&prepared) {
        Ok(projections) => projections,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shells::PreparedPostVerticalShellCache {
        predecessor,
        objects,
        caches,
    } = prepared;
    Ok(PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        caches,
        projections,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellProjection) {
    cleanup::successor(prepared);
}

fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(test)]
    {
        EVENTS.with(|events| events.borrow_mut().push(_step));
        let failed = FAILURE.with(|failure| match failure.get() {
            Some((step, remaining)) if step == _step && remaining == 1 => true,
            Some((step, remaining)) if step == _step => {
                failure.set(Some((step, remaining - 1)));
                false
            }
            _ => false,
        });
        if failed {
            return Err(SliceError::InvalidInput(
                "vertical-shell projection geometry is outside the supported Clipper range"
                    .to_owned(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_geometry_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    FAILURE.with(|failure| failure.set(None));
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    fail_geometry_at_occurrence(step, 1);
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at_occurrence(step: GeometryStep, occurrence: usize) {
    FAILURE.with(|failure| failure.set(Some((step, occurrence))));
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
