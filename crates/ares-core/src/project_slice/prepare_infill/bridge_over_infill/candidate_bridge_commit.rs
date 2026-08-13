#[cfg(test)]
mod tests;

use crate::geometry::Polygon;

use super::{
    candidate_bridge_postprocessing::PostprocessedCandidateBridge,
    types::{CandidateSource, CandidateSurface},
};

pub(in crate::project_slice) fn append_postprocessed_candidate(
    completed: &mut Vec<CandidateSurface>,
    source: CandidateSource,
    postprocessed: PostprocessedCandidateBridge,
) -> Vec<Polygon> {
    let PostprocessedCandidateBridge {
        boundary_polylines: _,
        bridging_area,
        bridging_angle,
        expansion_area,
    } = postprocessed;
    completed.push(CandidateSurface {
        source,
        new_polygons: bridging_area,
        bridge_angle: bridging_angle,
    });
    expansion_area
}

pub(in crate::project_slice) fn replace_candidate_layer(
    current: &mut Vec<CandidateSurface>,
    mut completed: Vec<CandidateSurface>,
) {
    std::mem::swap(current, &mut completed);
    completed.clear();
}
