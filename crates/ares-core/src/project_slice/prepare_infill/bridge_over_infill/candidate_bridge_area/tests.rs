mod errors;
mod geometry;
mod operation_order;

use super::{
    CandidateBridgeArea, CandidateBridgeAreaInput, prepare_candidate_bridge_area,
    prepare_candidate_bridge_area_using,
};
use crate::geometry::{ClipperError, Point, Polygon};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn operation_input<'a>(
    candidates: &'a [Polygon],
    deep: &'a [Polygon],
    unsupported: &'a [Polygon],
    expansion: &'a [Polygon],
    spacing: i64,
) -> CandidateBridgeAreaInput<'a> {
    CandidateBridgeAreaInput {
        candidate_polygons: candidates,
        deep_infill_area: deep,
        internal_unsupported_area: unsupported,
        expansion_area: expansion,
        scaled_spacing: spacing,
    }
}

fn prepare(
    candidates: &[Polygon],
    deep: &[Polygon],
    unsupported: &[Polygon],
    expansion: &[Polygon],
    spacing: i64,
) -> CandidateBridgeArea {
    prepare_candidate_bridge_area(candidates, deep, unsupported, expansion, spacing).unwrap()
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

fn snapshot(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
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

fn total_area(polygons: &[Polygon]) -> f64 {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .zip(polygon.points().iter().cycle().skip(1))
                .map(|(a, b)| (a.x() as f64 * b.y() as f64) - (b.x() as f64 * a.y() as f64))
                .sum::<f64>()
                .abs()
                * 0.5
        })
        .sum()
}

fn assert_range_error(result: Result<CandidateBridgeArea, ClipperError>) {
    assert!(matches!(result, Err(ClipperError::CoordinateOutOfRange)));
}
