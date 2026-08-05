mod cleanup;
mod filter;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shell_regularization};

pub(in crate::project_slice) use types::PreparedPostVerticalShellFiltering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    NeighborIntersection,
    ClosingGrow,
    ClosingShrink,
    CandidateScan,
    VisibilityDifference,
    CandidateExpansion,
    ProtectionDifference,
    EmptyGate,
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
    prepared: vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> Result<PreparedPostVerticalShellFiltering, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    let filters = match stage::filter(&prepared) {
        Ok(filters) => filters,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let vertical_shell_regularization::PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    } = prepared;
    Ok(PreparedPostVerticalShellFiltering {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellFiltering) {
    cleanup::successor(prepared);
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(test)]
    {
        EVENTS.with(|events| events.borrow_mut().push(_step));
        let failed = FAILURE.with(|failure| match failure.get() {
            Some((failed_step, remaining))
                if failed_step == _step && remaining == 1 && failure_site(_step) =>
            {
                true
            }
            Some((failed_step, remaining)) if failed_step == _step && failure_site(_step) => {
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

#[cfg(test)]
fn failure_site(step: GeometryStep) -> bool {
    matches!(
        step,
        GeometryStep::NeighborIntersection
            | GeometryStep::ClosingGrow
            | GeometryStep::ClosingShrink
            | GeometryStep::VisibilityDifference
            | GeometryStep::CandidateExpansion
            | GeometryStep::ProtectionDifference
    )
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
            .to_owned(),
    )
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_geometry_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    FAILURE.with(|failure| failure.set(None));
    filter::reset_volume_snapshots();
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    fail_geometry_at_occurrence(step, 1);
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at_occurrence(step: GeometryStep, occurrence: usize) {
    assert!(failure_site(step));
    assert!(occurrence > 0);
    FAILURE.with(|failure| failure.set(Some((step, occurrence))));
}

#[cfg(test)]
pub(in crate::project_slice) fn geometry_events() -> Vec<GeometryStep> {
    EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn threshold_bits(
    solid_infill_spacing: i64,
    scale: crate::geometry::CoordinateScale,
) -> [u64; 8] {
    let values = filter::thresholds(solid_infill_spacing, scale);
    [
        u64::from(values.minimum.to_bits()),
        values.scaled_small as u64,
        u64::from((values.scaled_small as f32).to_bits()),
        values.scaled_large as u64,
        u64::from((values.scaled_large as f32).to_bits()),
        u64::from(values.small.to_bits()),
        u64::from(values.large.to_bits()),
        values.epsilon_quotient.to_bits(),
    ]
}

#[cfg(test)]
pub(in crate::project_slice) fn epsilon_bits(scale: crate::geometry::CoordinateScale) -> u32 {
    (filter::thresholds(0, scale).epsilon_quotient as f32).to_bits()
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}
