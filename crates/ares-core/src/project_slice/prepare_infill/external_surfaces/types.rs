use crate::geometry::{ExPolygon, RegionExpansionEx, RegionExpansionParameters, WaveSeed};

pub(in crate::project_slice) struct ExpansionResult {
    pub(in crate::project_slice) anchors: Vec<WaveSeed>,
    pub(in crate::project_slice) expansions: Vec<RegionExpansionEx>,
}

pub(in crate::project_slice) struct ExpansionZone {
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) parameters: RegionExpansionParameters,
    pub(in crate::project_slice) expanded_into: bool,
}

impl ExpansionZone {
    pub(in crate::project_slice) fn new(
        expolygons: Vec<ExPolygon>,
        parameters: RegionExpansionParameters,
    ) -> Self {
        Self {
            expolygons,
            parameters,
            expanded_into: false,
        }
    }
}
