use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    prepare_infill::surface_type_detection::PreparedSurfaceTypeObject,
    region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) struct PreparedPostHorizontalShellPropagation {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
}

pub(super) struct WorkingProject {
    pub(super) objects: Vec<WorkingObject>,
}

pub(super) struct WorkingObject {
    pub(super) records: Option<Vec<Option<WorkingFillRecord>>>,
}

pub(super) struct WorkingFillRecord {
    pub(super) fill_surfaces: Vec<RegionSurface>,
    pub(super) dirty: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum SourceKind {
    Top,
    Bottom,
    BottomBridge,
}

impl SourceKind {
    pub(super) const fn surface_kind(self) -> RegionSurfaceKind {
        match self {
            Self::Top => RegionSurfaceKind::Top,
            Self::Bottom => RegionSurfaceKind::Bottom,
            Self::BottomBridge => RegionSurfaceKind::BottomBridge,
        }
    }
}

pub(super) const SOURCE_KINDS: [SourceKind; 3] = [
    SourceKind::Top,
    SourceKind::Bottom,
    SourceKind::BottomBridge,
];

pub(super) enum NeighborOutcome {
    EmptyIntersection,
    Rebuilt(Vec<RegionSurface>),
}
