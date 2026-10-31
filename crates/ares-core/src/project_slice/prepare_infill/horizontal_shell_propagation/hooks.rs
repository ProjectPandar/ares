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

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "horizontal-shell propagation geometry is outside the supported Clipper range".to_owned(),
    )
}

pub(super) fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

pub(super) fn record_event(_event: PropagationEvent) {}

pub(super) fn record_commit() {}

pub(super) fn record_invocation() {}

pub(super) fn record_disposal() {}
