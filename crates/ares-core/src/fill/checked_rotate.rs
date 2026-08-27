use crate::geometry::{ClipperError, Point};

const MIN_COORDINATE: f64 = i64::MIN as f64;
const MAX_COORDINATE_EXCLUSIVE: f64 = -MIN_COORDINATE;

// Source MultiPoint::rotate rounds with std::round (half away from zero), not
// Clipper's floor(x+0.5); negative half-values land one unit lower.
pub(super) fn checked_point(x: f64, y: f64) -> Result<Point, ClipperError> {
    Ok(Point::new(checked_round(x)?, checked_round(y)?))
}

fn checked_round(value: f64) -> Result<i64, ClipperError> {
    let rounded = value.round();
    if rounded.is_finite() && (MIN_COORDINATE..MAX_COORDINATE_EXCLUSIVE).contains(&rounded) {
        Ok(rounded as i64)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}

pub(super) fn rotate_point(point: Point, cosine: f64, sine: f64) -> Result<Point, ClipperError> {
    let x = point.x() as f64;
    let y = point.y() as f64;
    checked_point(cosine * x - sine * y, cosine * y + sine * x)
}

pub(super) fn rotate_points_with_trig(
    points: Vec<Point>,
    cosine: f64,
    sine: f64,
) -> Result<Vec<Point>, ClipperError> {
    points
        .into_iter()
        .map(|point| rotate_point(point, cosine, sine))
        .collect()
}

pub(super) fn rotate_points(points: Vec<Point>, angle: f64) -> Result<Vec<Point>, ClipperError> {
    rotate_points_with_trig(points, angle.cos(), angle.sin())
}
