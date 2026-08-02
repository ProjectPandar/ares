use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::classic::{
        perimeter_append::{AppendedPerimeterCollections, InactivePostCollectionBranches},
        traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice) struct PreparedPostClassicGapDomain {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedGapDomainObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapDomainObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedGapDomainRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapDomainRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedGapDomainSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedGapDomainSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) inactive: InactivePostCollectionBranches,
    pub(in crate::project_slice) appended: AppendedPerimeterCollections,
    pub(in crate::project_slice) pre_medial: Option<PreMedialGapDomain>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreMedialGapDomain {
    pub(in crate::project_slice) min: f64,
    pub(in crate::project_slice) max: f64,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
}
