use crate::geometry::{
    ClipperError, JoinType, Polygon, Polyline, difference_polygons_paths,
    intersection_polygons_paths, offset_paths, opening_paths, union_polygons_paths_pair,
};

use super::{
    anchored_polygon::{BridgeSampling, construct_anchored_polygon},
    candidate_anchored_bridge::to_lines,
    candidate_collision_reconstruction::CollisionResolvedCandidateBridge,
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PostprocessedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
    pub(in crate::project_slice) expansion_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn postprocess_candidate_bridge(
    collision_resolved: CollisionResolvedCandidateBridge,
    expansion_area: Vec<Polygon>,
    (limiting_area, total_fill_area, total_top_area): (&[Polygon], &[Polygon], &[Polygon]),
    sampling: BridgeSampling,
) -> Result<PostprocessedCandidateBridge, ClipperError> {
    postprocess_candidate_bridge_using(
        collision_resolved,
        expansion_area,
        (limiting_area, total_fill_area, total_top_area),
        sampling,
        (
            |paths, delta| opening_paths(paths, delta, delta, JoinType::Miter, 3.0),
            |paths, delta| {
                let expanded = offset_paths(paths, delta, JoinType::Miter, 3.0)?;
                offset_paths(&expanded, -delta, JoinType::Miter, 3.0)
            },
            intersection_polygons_paths,
            difference_polygons_paths,
        ),
    )
}

fn postprocess_candidate_bridge_using<Opening, Closing, Intersection, Difference>(
    collision_resolved: CollisionResolvedCandidateBridge,
    expansion_area: Vec<Polygon>,
    (limiting_area, total_fill_area, total_top_area): (&[Polygon], &[Polygon], &[Polygon]),
    sampling: BridgeSampling,
    (opening, closing, mut intersection, mut difference): (
        Opening,
        Closing,
        Intersection,
        Difference,
    ),
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
    let opening_delta = (sampling.spacing as f64 * 0.75_f64) as f32;
    let closing_delta = sampling.spacing as f32;

    let bridging_area = opening(&bridging_area, opening_delta)?;
    let mut bridging_area = closing(&bridging_area, closing_delta)?;
    // PrintObject.cpp:3998–4003 restores contacts without resmoothing them.
    if let Some(overlap) = sampling.restore_overlap
        && !bridging_area.is_empty()
    {
        let restored = construct_anchored_polygon(
            &bridging_area,
            &to_lines(&boundary_polylines),
            bridging_angle,
            sampling,
            overlap,
        )?;
        bridging_area = union_polygons_paths_pair(&bridging_area, &restored)?;
    }
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
