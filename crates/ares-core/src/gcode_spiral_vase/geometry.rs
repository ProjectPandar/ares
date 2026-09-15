//! Spiral-vase geometry helpers — polyline length, point projection.

use crate::{LayerExtrusionMoves, ToolpathMoveKind};

pub(super) type Point2 = crate::Point2;

pub(super) fn total_print_xy(layer: &LayerExtrusionMoves) -> f64 {
    let mut previous = None;
    let mut total = 0.0;
    for move_ in layer.moves() {
        let point = move_.point();
        if move_.kind() == ToolpathMoveKind::Print
            && let Some(start) = previous
        {
            total += distance(start, point);
        }
        previous = Some(point);
    }
    total
}

pub(super) fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

pub(super) fn distance_f32(start: [f32; 2], end: [f32; 2]) -> f32 {
    ((end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2)).sqrt()
}

pub(super) fn nearest_point_on_polyline(points: &[[f32; 2]], point: [f32; 2]) -> Option<[f32; 2]> {
    points
        .windows(2)
        .map(|segment| project_point_to_segment(segment[0], segment[1], point))
        .min_by(|left, right| distance_f32(*left, point).total_cmp(&distance_f32(*right, point)))
}

pub(super) fn project_point_to_segment(
    start: [f32; 2],
    end: [f32; 2],
    point: [f32; 2],
) -> [f32; 2] {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return start;
    }
    let t = (((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / length_squared)
        .clamp(0.0, 1.0);
    [start[0] + dx * t, start[1] + dy * t]
}
