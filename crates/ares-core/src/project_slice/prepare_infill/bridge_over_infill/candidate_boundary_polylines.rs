#[cfg(test)]
mod tests;

use super::candidate_bridge_area::CandidateBridgeArea;
use crate::geometry::{ClipperError, Coord, JoinType, Polygon, Polyline, offset_paths};

const MITER_LIMIT: f64 = 3.0;

struct CandidateBoundaryInput<'a> {
    candidate_area: &'a CandidateBridgeArea,
    total_fill_area: &'a [Polygon],
    scaled_spacing: Coord,
    spacing: f32,
}

pub(in crate::project_slice) fn prepare_candidate_boundary_polylines(
    candidate_area: &CandidateBridgeArea,
    total_fill_area: &[Polygon],
    scaled_spacing: Coord,
    spacing: f32,
) -> Result<Option<Vec<Polyline>>, ClipperError> {
    prepare_candidate_boundary_polylines_using(
        CandidateBoundaryInput {
            candidate_area,
            total_fill_area,
            scaled_spacing,
            spacing,
        },
        |paths, delta| offset_paths(paths, delta, JoinType::Miter, MITER_LIMIT),
    )
}

fn prepare_candidate_boundary_polylines_using<Offset>(
    input: CandidateBoundaryInput<'_>,
    mut offset: Offset,
) -> Result<Option<Vec<Polyline>>, ClipperError>
where
    Offset: FnMut(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateBoundaryInput {
        candidate_area,
        total_fill_area,
        scaled_spacing,
        spacing,
    } = input;
    if candidate_area.area_to_be_bridge.is_empty() {
        return Ok(None);
    }

    let total_delta = (1.3_f64 * scaled_spacing as f64) as f32;
    debug_assert!(total_delta > 0.0);
    let total_boundaries = offset(total_fill_area, total_delta)?;
    let mut boundaries = into_closed_polylines(total_boundaries);

    let limiting_delta = (0.3_f64 * f64::from(spacing)) as f32;
    debug_assert!(limiting_delta > 0.0);
    let limiting_boundaries = offset(&candidate_area.limiting_area, limiting_delta)?;
    boundaries.extend(into_closed_polylines(limiting_boundaries));
    Ok(Some(boundaries))
}

fn into_closed_polylines(polygons: Vec<Polygon>) -> Vec<Polyline> {
    polygons
        .into_iter()
        .map(|polygon| {
            let mut points = polygon.into_points();
            let first = points[0];
            points.push(first);
            Polyline::new(points)
        })
        .collect()
}
