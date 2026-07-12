use std::collections::BTreeMap;

use serde_json::Value;

use super::support_rectangle::{EPSILON, RectangleBounds};
use crate::{Point2, SliceError};

const SUPPORT_ANGLE: &str = "support_angle";
const DEGREES_PER_TURN: f64 = 360.0;

#[derive(Clone, Copy)]
struct Vector2 {
    x: f64,
    y: f64,
}

pub(crate) fn parse_support_angle(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get(SUPPORT_ANGLE) else {
        return Ok(0.0);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|angle: &f64| angle.is_finite() && (0.0..=359.0).contains(angle))
    .ok_or_else(invalid_support_angle)
}

pub(crate) fn rotated_rectangle_lines(
    bounds: RectangleBounds,
    pitch: f64,
    angle_degrees: f64,
) -> Vec<[Point2; 2]> {
    let angle = angle_degrees.rem_euclid(DEGREES_PER_TURN).to_radians();
    let direction = Vector2 {
        x: angle.cos(),
        y: angle.sin(),
    };
    let mut normal = Vector2 {
        x: -direction.y,
        y: direction.x,
    };
    if normal.y < 0.0 || (normal.y.abs() <= EPSILON && normal.x < 0.0) {
        normal.x = -normal.x;
        normal.y = -normal.y;
    }

    let corners = [
        Point2::new(bounds.min_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.max_y),
        Point2::new(bounds.min_x, bounds.max_y),
    ];
    let min_offset = corners
        .iter()
        .map(|point| project(*point, normal))
        .fold(f64::INFINITY, f64::min);
    let max_offset = corners
        .iter()
        .map(|point| project(*point, normal))
        .fold(f64::NEG_INFINITY, f64::max);

    let mut lines = Vec::new();
    let mut offset = min_offset;
    while offset <= max_offset + EPSILON {
        if let Some(line) = clipped_line(bounds, direction, normal, offset) {
            lines.push(line);
        }
        let next_offset = offset + pitch;
        if next_offset <= offset {
            break;
        }
        offset = next_offset;
    }
    lines
}

fn clipped_line(
    bounds: RectangleBounds,
    direction: Vector2,
    normal: Vector2,
    offset: f64,
) -> Option<[Point2; 2]> {
    let mut points = Vec::new();
    push_vertical_intersection(&mut points, bounds.min_x, bounds, normal, offset);
    push_vertical_intersection(&mut points, bounds.max_x, bounds, normal, offset);
    push_horizontal_intersection(&mut points, bounds.min_y, bounds, normal, offset);
    push_horizontal_intersection(&mut points, bounds.max_y, bounds, normal, offset);

    points.sort_by(|left, right| project(*left, direction).total_cmp(&project(*right, direction)));
    let first = points.first().copied()?;
    let last = points.last().copied()?;
    (distance_squared(first, last) > EPSILON * EPSILON).then_some([first, last])
}

fn push_vertical_intersection(
    points: &mut Vec<Point2>,
    x: f64,
    bounds: RectangleBounds,
    normal: Vector2,
    offset: f64,
) {
    if normal.y.abs() <= EPSILON {
        return;
    }
    let y = (offset - x * normal.x) / normal.y;
    if y >= bounds.min_y - EPSILON && y <= bounds.max_y + EPSILON {
        push_unique(points, Point2::new(x, y.clamp(bounds.min_y, bounds.max_y)));
    }
}

fn push_horizontal_intersection(
    points: &mut Vec<Point2>,
    y: f64,
    bounds: RectangleBounds,
    normal: Vector2,
    offset: f64,
) {
    if normal.x.abs() <= EPSILON {
        return;
    }
    let x = (offset - y * normal.y) / normal.x;
    if x >= bounds.min_x - EPSILON && x <= bounds.max_x + EPSILON {
        push_unique(points, Point2::new(x.clamp(bounds.min_x, bounds.max_x), y));
    }
}

fn push_unique(points: &mut Vec<Point2>, point: Point2) {
    if !points.iter().any(|existing| points_close(*existing, point)) {
        points.push(point);
    }
}

fn points_close(left: Point2, right: Point2) -> bool {
    distance_squared(left, right) <= EPSILON * EPSILON
}

fn distance_squared(left: Point2, right: Point2) -> f64 {
    let dx = left.x() - right.x();
    let dy = left.y() - right.y();
    dx * dx + dy * dy
}

fn project(point: Point2, vector: Vector2) -> f64 {
    point.x() * vector.x + point.y() * vector.y
}

fn invalid_support_angle() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_ANGLE} must be a finite number from 0 through 359 degrees"
    ))
}
