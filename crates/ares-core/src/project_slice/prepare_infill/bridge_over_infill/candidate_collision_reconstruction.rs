use crate::geometry::{
    ClipperError, JoinType, Line, Polygon, Polyline, intersection_polygons_paths, offset_paths,
};

use super::{
    anchored_polygon::{BridgeSampling, construct_anchored_polygon},
    candidate_anchored_bridge::{CandidateAnchoredBridge, to_lines},
    types::CandidateSurface,
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct CollisionResolvedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
}

pub(in crate::project_slice) fn reconstruct_candidate_bridge_collision(
    area_to_be_bridge: &[Polygon],
    initial: CandidateAnchoredBridge,
    sampling_and_angle: (BridgeSampling, f64),
    completed_surfaces: &[CandidateSurface],
) -> Result<CollisionResolvedCandidateBridge, ClipperError> {
    reconstruct_candidate_bridge_collision_using(
        area_to_be_bridge,
        initial,
        sampling_and_angle,
        completed_surfaces,
        (
            |polygons, delta| offset_paths(polygons, delta, JoinType::Miter, 3.0),
            intersection_polygons_paths,
            construct_anchored_polygon,
        ),
    )
}

fn reconstruct_candidate_bridge_collision_using<Expand, Intersect, Construct>(
    area_to_be_bridge: &[Polygon],
    initial: CandidateAnchoredBridge,
    (sampling, bridging_angle): (BridgeSampling, f64),
    completed_surfaces: &[CandidateSurface],
    (expand, mut intersection, construct): (Expand, Intersect, Construct),
) -> Result<CollisionResolvedCandidateBridge, ClipperError>
where
    Expand: FnOnce(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Intersect: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Construct:
        FnOnce(&[Polygon], &[Line], f64, BridgeSampling, i64) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateAnchoredBridge {
        boundary_polylines,
        bridging_area,
    } = initial;
    let delta = (3.0_f64 * sampling.spacing as f64) as f32;
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
            construct(area_to_be_bridge, &lines, selected_angle, sampling, 0)?,
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
