use crate::geometry::ExPolygon;
use crate::project_slice::perimeters::classic::{
    medial_gap::MedialGapDomain,
    perimeter_append::{AppendedPerimeterCollections, InactivePostCollectionBranches},
    traversal::PreparedPostClassicTraversal,
};

use super::GapFillCollection;

pub(in crate::project_slice) struct PreparedPostClassicGapExtrusion {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedGapExtrusionObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapExtrusionObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedGapExtrusionRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapExtrusionRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedGapExtrusionSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapExtrusionSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) inactive: InactivePostCollectionBranches,
    pub(in crate::project_slice) appended: AppendedPerimeterCollections,
    pub(in crate::project_slice) medial: Option<MedialGapDomain>,
    pub(in crate::project_slice) gap_fill: GapFillCollection,
    pub(in crate::project_slice) remaining: Vec<ExPolygon>,
}
