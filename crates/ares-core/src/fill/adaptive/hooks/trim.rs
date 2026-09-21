//! Polyline length and endpoint trimming (`FillAdaptive.cpp:993-1000`).

use super::geometry::length;
use crate::geometry::{Line, Point};

pub(super) fn polyline_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| length(&Line::new(pair[0], pair[1])))
        .sum()
}

/// `:993-1000` — move `points`' front (or back) endpoint inward by
/// `1.155·trim` toward the current opposite endpoint (truncating casts).
pub(super) fn trim_endpoint(
    points: &mut [Point],
    front: bool,
    active: bool,
    scaled_trim_distance: f64,
) {
    if !active || points.len() < 2 {
        return;
    }
    let endpoint = if front {
        points[0]
    } else {
        points[points.len() - 1]
    };
    let other = if front {
        points[points.len() - 1]
    } else {
        points[0]
    };
    let dx = (other.x() - endpoint.x()) as f64;
    let dy = (other.y() - endpoint.y()) as f64;
    let n = (dx * dx + dy * dy).sqrt();
    if n <= 0.0 {
        return;
    }
    let step = 1.155 * scaled_trim_distance;
    let mx = (dx / n * step) as i64;
    let my = (dy / n * step) as i64;
    let moved = Point::new(endpoint.x() + mx, endpoint.y() + my);
    if front {
        points[0] = moved;
    } else {
        let last = points.len() - 1;
        points[last] = moved;
    }
}
