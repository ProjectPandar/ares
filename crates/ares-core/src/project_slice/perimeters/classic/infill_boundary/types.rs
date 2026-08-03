use crate::{
    geometry::{Coord, ExPolygon},
    project_slice::{
        perimeters::classic::{
            gap_extrusion::PreparedGapExtrusionSurface, traversal::PreparedPostClassicTraversal,
        },
        region_slices::RegionSurface,
    },
};

pub(in crate::project_slice) struct PreparedPostClassicInfillBoundary {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedInfillBoundaryObject>,
}

pub(in crate::project_slice) struct PreparedInfillBoundaryObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedInfillBoundaryRecord>>,
}

pub(in crate::project_slice) struct PreparedInfillBoundaryRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedGapExtrusionSurface>,
    pub(in crate::project_slice) fill_surfaces: Vec<RegionSurface>,
    pub(in crate::project_slice) fill_no_overlap: Vec<ExPolygon>,
    pub(in crate::project_slice) overlap: Vec<InfillBoundaryOverlap>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct InfillBoundaryOverlap {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) inset: Coord,
    pub(in crate::project_slice) infill_peri_overlap: Coord,
    pub(in crate::project_slice) top_infill_peri_overlap: Coord,
    pub(in crate::project_slice) min_perimeter_infill_spacing: Coord,
    pub(in crate::project_slice) scaled_resolution: f64,
}

pub(super) struct ValidatedProject {
    pub(super) objects: Vec<ValidatedObject>,
}

pub(super) struct ValidatedObject {
    pub(super) records: Vec<Option<ValidatedRecord>>,
}

pub(super) struct ValidatedRecord {
    pub(super) surfaces: Vec<ValidatedSurface>,
}

#[derive(Clone, Copy)]
pub(super) struct ValidatedSurface {
    pub(super) overlap: InfillBoundaryOverlap,
    pub(super) ordinary_first: f32,
    pub(super) ordinary_second: f32,
    pub(super) top_offset: f32,
    pub(super) top_overlap: f32,
    pub(super) no_overlap: NoOverlapOffset,
}

#[derive(Clone, Copy)]
pub(super) enum NoOverlapOffset {
    Two { first: f32, second: f32 },
    One { delta: f32 },
}

pub(super) struct StagedObject {
    pub(super) records: Vec<Option<StagedRecord>>,
}

pub(super) struct StagedRecord {
    pub(super) surface_count: usize,
    pub(super) fill_surfaces: Vec<RegionSurface>,
    pub(super) fill_no_overlap: Vec<ExPolygon>,
    pub(super) overlap: Vec<InfillBoundaryOverlap>,
}
