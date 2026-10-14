//! `ExtrusionEntity.cpp:75–84::polygons_covered_by_spacing`.
use crate::geometry::{ClipperError, CoordinateScale, JoinType, Point, Polygon, offset_open_paths};

pub(in crate::project_slice::gcode_emit) fn covered_by_spacing(
    points: impl IntoIterator<Item = Point>,
    spacing: f32,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let spacing = (f64::from(spacing) / scale.factor()) as i64;
    offset_open_paths(
        &[Polygon::new(points.into_iter().collect())],
        0.5 * spacing as f32 + epsilon,
        JoinType::Square,
        0.0,
    )
}
