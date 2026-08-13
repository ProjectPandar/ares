mod arithmetic;
mod errors;
mod gate;
mod operation_order;
mod output;

use super::{
    CandidateBoundaryInput, prepare_candidate_boundary_polylines,
    prepare_candidate_boundary_polylines_using,
};
use crate::geometry::{ClipperError, Point, Polygon, Polyline};
use crate::project_slice::prepare_infill::bridge_over_infill::candidate_bridge_area::CandidateBridgeArea;

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn candidate_area(survivors: Vec<Polygon>, limiting: Vec<Polygon>) -> CandidateBridgeArea {
    CandidateBridgeArea {
        area_to_be_bridge: survivors,
        limiting_area: limiting,
    }
}

fn operation_input<'a>(
    area: &'a CandidateBridgeArea,
    total: &'a [Polygon],
    scaled_spacing: i64,
    spacing: f32,
) -> CandidateBoundaryInput<'a> {
    CandidateBoundaryInput {
        candidate_area: area,
        total_fill_area: total,
        scaled_spacing,
        spacing,
    }
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn outside_range() -> Polygon {
    rectangle(HI_RANGE, 0, HI_RANGE + 10, 10)
}

fn snapshot_polygons(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

fn snapshot_polylines(polylines: &[Polyline]) -> Vec<Vec<(i64, i64)>> {
    polylines
        .iter()
        .map(|polyline| {
            polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

fn assert_range_error(result: Result<Option<Vec<Polyline>>, ClipperError>) {
    assert!(matches!(result, Err(ClipperError::CoordinateOutOfRange)));
}
