#[cfg(test)]
mod tests;

use crate::{
    geometry::{
        ClipperError, CoordinateScale, JoinType, Line, Polygon, Polyline,
        intersection_polygons_paths, offset_paths,
    },
    project_slice::perimeters::types::Flow,
};

use super::{
    anchored_polygon::{construct_anchored_polygon, scaled_flow_value},
    candidate_anchored_bridge::{CandidateAnchoredBridge, to_lines},
    types::CandidateSurface,
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct CollisionResolvedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source collision block inputs without inventing ownership"
)]
pub(in crate::project_slice) fn reconstruct_candidate_bridge_collision(
    area_to_be_bridge: &[Polygon],
    initial: CandidateAnchoredBridge,
    bridging_flow: Flow,
    bridging_angle: f64,
    completed_surfaces: &[CandidateSurface],
    scale: CoordinateScale,
) -> Result<CollisionResolvedCandidateBridge, ClipperError> {
    reconstruct_candidate_bridge_collision_using(
        area_to_be_bridge,
        initial,
        bridging_flow,
        bridging_angle,
        completed_surfaces,
        scale,
        |polygons, delta| offset_paths(polygons, delta, JoinType::Miter, 3.0),
        intersection_polygons_paths,
        construct_anchored_polygon,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps the source operands and injected geometry operations explicit"
)]
fn reconstruct_candidate_bridge_collision_using<Expand, Intersect, Construct>(
    area_to_be_bridge: &[Polygon],
    initial: CandidateAnchoredBridge,
    bridging_flow: Flow,
    bridging_angle: f64,
    completed_surfaces: &[CandidateSurface],
    scale: CoordinateScale,
    expand: Expand,
    mut intersection: Intersect,
    construct: Construct,
) -> Result<CollisionResolvedCandidateBridge, ClipperError>
where
    Expand: FnOnce(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Intersect: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Construct: FnOnce(
        &[Polygon],
        &[Line],
        Flow,
        f64,
        CoordinateScale,
    ) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateAnchoredBridge {
        boundary_polylines,
        bridging_area,
    } = initial;
    let scaled_spacing = scaled_flow_value(bridging_flow.spacing, scale);
    let delta = (3.0_f64 * scaled_spacing as f64) as f32;
    let expanded_area = expand(&bridging_area, delta)?;

    let mut selected_angle = None;
    for surface in completed_surfaces {
        if !intersection(&surface.new_polygons, &expanded_area)?.is_empty() {
            selected_angle = Some(surface.bridge_angle);
            break;
        }
    }

    let (bridging_area, bridging_angle) = if let Some(selected_angle) = selected_angle {
        let lines = to_lines(&boundary_polylines);
        (
            construct(
                area_to_be_bridge,
                &lines,
                bridging_flow,
                selected_angle,
                scale,
            )?,
            selected_angle,
        )
    } else {
        (bridging_area, bridging_angle)
    };

    Ok(CollisionResolvedCandidateBridge {
        boundary_polylines,
        bridging_area,
        bridging_angle,
    })
}
