mod cleanup;
mod regularize;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_trimming};

pub(in crate::project_slice) use types::PreparedPostVerticalShellRegularization;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    Union,
    Offset2First,
    Offset2Second,
    Shrink,
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
    prepared: vertical_shell_trimming::PreparedPostVerticalShellTrim,
) -> Result<PreparedPostVerticalShellRegularization, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    let regularizations = match stage::regularize(&prepared) {
        Ok(regularizations) => regularizations,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shell_trimming::PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        caches,
        projections,
        trims,
    } = prepared;
    Ok(PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellRegularization) {
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
        "vertical-shell regularization geometry is outside the supported Clipper range".to_owned(),
    )
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
pub(in crate::project_slice) fn radii_bits(solid_infill_spacing: i64) -> [u32; 7] {
    let minimum = regularize::min_perimeter_infill_spacing(solid_infill_spacing);
    let radii = regularize::radii(solid_infill_spacing);
    [
        minimum.to_bits(),
        radii.narrow_ensure.to_bits(),
        radii.narrow_sparse.to_bits(),
        radii.tiny_overlap.to_bits(),
        (-radii.narrow_ensure).to_bits(),
        (radii.narrow_ensure + radii.narrow_sparse).to_bits(),
        (-(radii.narrow_sparse - radii.tiny_overlap)).to_bits(),
    ]
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}
