use std::collections::BTreeMap;

use crate::geometry::Polygon;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct CandidateSource {
    pub(in crate::project_slice) layer_index: usize,
    pub(in crate::project_slice) region_index: usize,
    pub(in crate::project_slice) surface_index: usize,
}

pub(in crate::project_slice) struct CandidateSurface {
    pub(in crate::project_slice) source: CandidateSource,
    pub(in crate::project_slice) new_polygons: Vec<Polygon>,
    pub(in crate::project_slice) bridge_angle: f64,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "retained for the next bridge-over-infill source slice"
    )
)]
pub(in crate::project_slice) struct BridgeCandidateObject {
    pub(in crate::project_slice) has_lightning_infill: bool,
    pub(in crate::project_slice) surfaces_by_layer: BTreeMap<usize, Vec<CandidateSurface>>,
}
