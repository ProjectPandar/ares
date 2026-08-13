#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the source-cited dependency remains unwired until the bridge transaction"
    )
)]

#[cfg(test)]
mod tests;

use crate::{
    geometry::{
        ClipperError, CoordinateScale, JoinType, Polygon, Polyline, difference_polygons_paths,
        intersection_polygons_paths, offset_paths, opening_paths,
    },
    project_slice::perimeters::types::Flow,
};

use super::{
    anchored_polygon::scaled_flow_value,
    candidate_collision_reconstruction::CollisionResolvedCandidateBridge,
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PostprocessedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
    pub(in crate::project_slice) expansion_area: Vec<Polygon>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source postprocessing operands without inventing ownership"
)]
pub(in crate::project_slice) fn postprocess_candidate_bridge(
    collision_resolved: CollisionResolvedCandidateBridge,
    expansion_area: Vec<Polygon>,
    limiting_area: &[Polygon],
    total_fill_area: &[Polygon],
    total_top_area: &[Polygon],
    bridging_flow: Flow,
    scale: CoordinateScale,
) -> Result<PostprocessedCandidateBridge, ClipperError> {
    postprocess_candidate_bridge_using(
        collision_resolved,
        expansion_area,
        limiting_area,
        total_fill_area,
        total_top_area,
        bridging_flow,
        scale,
        |paths, delta| opening_paths(paths, delta, delta, JoinType::Miter, 3.0),
        |paths, delta| {
            let expanded = offset_paths(paths, delta, JoinType::Miter, 3.0)?;
            offset_paths(&expanded, -delta, JoinType::Miter, 3.0)
        },
        intersection_polygons_paths,
        difference_polygons_paths,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps the source operands and injected geometry operations explicit"
)]
fn postprocess_candidate_bridge_using<Opening, Closing, Intersection, Difference>(
    collision_resolved: CollisionResolvedCandidateBridge,
    expansion_area: Vec<Polygon>,
    limiting_area: &[Polygon],
    total_fill_area: &[Polygon],
    total_top_area: &[Polygon],
    bridging_flow: Flow,
    scale: CoordinateScale,
    opening: Opening,
    closing: Closing,
    mut intersection: Intersection,
    mut difference: Difference,
) -> Result<PostprocessedCandidateBridge, ClipperError>
where
    Opening: FnOnce(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Closing: FnOnce(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Intersection: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Difference: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
{
    let CollisionResolvedCandidateBridge {
        boundary_polylines,
        bridging_area,
        bridging_angle,
    } = collision_resolved;
    let scaled_spacing = scaled_flow_value(bridging_flow.spacing, scale);
    let opening_delta = (scaled_spacing as f64 * 0.75_f64) as f32;
    let closing_delta = scaled_spacing as f32;

    let bridging_area = opening(&bridging_area, opening_delta)?;
    let bridging_area = closing(&bridging_area, closing_delta)?;
    let bridging_area = intersection(&bridging_area, limiting_area)?;
    let bridging_area = intersection(&bridging_area, total_fill_area)?;
    let bridging_area = difference(&bridging_area, total_top_area)?;
    let expansion_area = difference(&expansion_area, &bridging_area)?;

    Ok(PostprocessedCandidateBridge {
        boundary_polylines,
        bridging_area,
        bridging_angle,
        expansion_area,
    })
}
