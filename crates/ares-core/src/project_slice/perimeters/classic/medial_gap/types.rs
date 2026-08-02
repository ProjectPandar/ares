use crate::{
    geometry::ThickPolyline,
    project_slice::perimeters::classic::{
        gap_domain::PreMedialGapDomain,
        perimeter_append::{AppendedPerimeterCollections, InactivePostCollectionBranches},
        traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice) struct PreparedPostClassicMedialGap {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedMedialGapObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedMedialGapObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedMedialGapRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedMedialGapRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedMedialGapSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedMedialGapSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) inactive: InactivePostCollectionBranches,
    pub(in crate::project_slice) appended: AppendedPerimeterCollections,
    pub(in crate::project_slice) medial: Option<MedialGapDomain>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct MedialGapDomain {
    pub(in crate::project_slice) predecessor: PreMedialGapDomain,
    pub(in crate::project_slice) polylines: Vec<ThickPolyline>,
}
