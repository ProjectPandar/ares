use super::SourceKind;
use crate::SliceError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PropagationEvent {
    FillClone {
        object: usize,
        layer: usize,
    },
    RecordVisit {
        object: usize,
        layer: usize,
    },
    EnsureAllSkip {
        object: usize,
        layer: usize,
    },
    SourceKindVisit {
        object: usize,
        layer: usize,
        kind: SourceKind,
    },
    NeighborVisit {
        object: usize,
        source: usize,
        neighbor: usize,
        kind: SourceKind,
    },
    Rebuild {
        object: usize,
        source: usize,
        neighbor: usize,
        kind: SourceKind,
    },
    DirtyCommit {
        object: usize,
        layer: usize,
    },
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct GatherObservation {
    pub(in crate::project_slice) object: usize,
    pub(in crate::project_slice) layer: usize,
    pub(in crate::project_slice) kind: SourceKind,
    pub(in crate::project_slice) dirty_before_gather: bool,
    pub(in crate::project_slice) path_count: usize,
    pub(in crate::project_slice) path_digest: i128,
    pub(in crate::project_slice) original_path_digest: i128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    SafetyIntersection,
    NeighborExternalWidthScale,
    FirstOpeningShrink,
    FirstOpeningExpand,
    FirstTooNarrowDifference,
    FirstTrimDifference,
    SourceSolidWidthScale,
    SecondOpeningShrink,
    SecondOpeningExpand,
    SecondTooNarrowDifference,
    RepairExpansion,
    RepairIntersection,
    SolidUnion,
    InternalSafetyDifference,
    ExternalGroupDifference,
}

#[cfg(test)]
thread_local! {
    static EVENTS: std::cell::RefCell<Vec<PropagationEvent>> = const { std::cell::RefCell::new(Vec::new()) };
    static GEOMETRY_EVENTS: std::cell::RefCell<Vec<GeometryStep>> = const { std::cell::RefCell::new(Vec::new()) };
    static GATHERS: std::cell::RefCell<Vec<GatherObservation>> = const { std::cell::RefCell::new(Vec::new()) };
    static FAILURE: std::cell::Cell<Option<(GeometryStep, usize)>> = const { std::cell::Cell::new(None) };
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static COMMITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static ROLLBACK_SNAPSHOTS: std::cell::RefCell<Vec<i128>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "horizontal-shell propagation geometry is outside the supported Clipper range".to_owned(),
    )
}

pub(super) fn geometry_step(step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(not(test))]
    let _ = step;
    #[cfg(test)]
    {
        GEOMETRY_EVENTS.with(|events| events.borrow_mut().push(step));
        let failed = FAILURE.with(|failure| match failure.get() {
            Some((selected, remaining)) if selected == step && remaining == 1 => true,
            Some((selected, remaining)) if selected == step => {
                failure.set(Some((selected, remaining - 1)));
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

pub(super) fn record_event(event: PropagationEvent) {
    #[cfg(test)]
    EVENTS.with(|events| events.borrow_mut().push(event));
    #[cfg(not(test))]
    let _ = event;
}

#[cfg(test)]
pub(super) fn record_gather(observation: GatherObservation) {
    GATHERS.with(|gathers| gathers.borrow_mut().push(observation));
}

#[cfg(test)]
pub(super) fn path_digest(paths: &[crate::geometry::Polygon]) -> i128 {
    let mut digest = 0x004f_2653_4552_4941_4c5f_4741_5448_4552_i128;
    for path in paths {
        digest = digest.wrapping_mul(1099511628211).wrapping_add(1);
        for point in path.points() {
            digest = digest
                .wrapping_mul(1099511628211)
                .wrapping_add(i128::from(point.x()));
            digest = digest
                .wrapping_mul(1099511628211)
                .wrapping_add(i128::from(point.y()));
        }
    }
    digest
}

pub(super) fn record_commit() {
    #[cfg(test)]
    COMMITS.with(|count| count.set(count.get() + 1));
}

pub(super) fn record_invocation() {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
}

pub(super) fn record_disposal() {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    GEOMETRY_EVENTS.with(|events| events.borrow_mut().clear());
    GATHERS.with(|gathers| gathers.borrow_mut().clear());
    FAILURE.with(|failure| failure.set(None));
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
    COMMITS.with(|count| count.set(0));
    ROLLBACK_SNAPSHOTS.with(|snapshots| snapshots.borrow_mut().clear());
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    FAILURE.with(|failure| failure.set(Some((step, 1))));
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at_occurrence(step: GeometryStep, occurrence: usize) {
    assert!(occurrence > 0);
    FAILURE.with(|failure| failure.set(Some((step, occurrence))));
}

#[cfg(test)]
pub(in crate::project_slice) fn events() -> Vec<PropagationEvent> {
    EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn geometry_events() -> Vec<GeometryStep> {
    GEOMETRY_EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn gather_observations() -> Vec<GatherObservation> {
    GATHERS.with(|gathers| gathers.borrow().clone())
}

#[cfg(test)]
pub(super) fn record_rollback_snapshot(snapshot: i128) {
    ROLLBACK_SNAPSHOTS.with(|snapshots| snapshots.borrow_mut().push(snapshot));
}

#[cfg(test)]
pub(in crate::project_slice) fn rollback_snapshots() -> Vec<i128> {
    ROLLBACK_SNAPSHOTS.with(|snapshots| snapshots.borrow().clone())
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn commits() -> usize {
    COMMITS.with(std::cell::Cell::get)
}
