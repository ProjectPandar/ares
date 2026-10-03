//! `raft::fills` geometry helpers, split from `fills.rs` for the <400 LOC rule.

use crate::fill::rectilinear::segments::{IntersectionKind, SegmentedLine};
use crate::geometry::{ClipperError, CoordinateScale, Point, Polyline};

pub(super) fn clip_polyline_end(polyline: &mut Polyline, distance: f64) {
    let mut points = polyline.clone().into_points();
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points.len() - 1;
        let (dx, dy) = (
            points[last].x() - points[last - 1].x(),
            points[last].y() - points[last - 1].y(),
        );
        let length = ((dx * dx + dy * dy) as f64).sqrt();
        if length > remaining {
            // `Polyline::clip_end` (`Polyline.cpp:56-72`): new endpoint
            // `last + (prev - last) * (remaining / len)`, truncated
            // toward zero by Eigen `.cast<coord_t>()`.
            let (vx, vy) = (
                (points[last - 1].x() - points[last].x()) as f64,
                (points[last - 1].y() - points[last].y()) as f64,
            );
            let lsqr = vx * vx + vy * vy;
            let factor = remaining / lsqr.sqrt();
            points[last] = Point::new(
                (points[last].x() as f64 + vx * factor) as i64,
                (points[last].y() as f64 + vy * factor) as i64,
            );
            *polyline = Polyline::new(points);
            return;
        }
        remaining -= length;
        points.pop();
    }
    *polyline = Polyline::new(points);
}
pub(super) fn vertical_segments(line: &SegmentedLine) -> Vec<Polyline> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < line.intersections.len() {
        let low = &line.intersections[index];
        if low.kind != IntersectionKind::OuterLow {
            index += 1;
            continue;
        }
        let Some(high) = line.intersections.get(index + 1) else {
            break;
        };
        if high.kind == IntersectionKind::OuterHigh {
            out.push(Polyline::new(vec![low.point, high.point]));
            index += 2;
        } else {
            index += 1;
        }
    }
    out
}

pub(super) fn align_to_grid(coordinate: i64, spacing: i64, base: i64) -> Result<i64, ClipperError> {
    let spacing = i128::from(spacing);
    let delta = i128::from(coordinate) - i128::from(base);
    i64::try_from(i128::from(base) + delta.div_euclid(spacing) * spacing)
        .map_err(|_| ClipperError::CoordinateOutOfRange)
}

pub(super) fn checked_scale_f32(scale: CoordinateScale, value: f64) -> Result<f32, ClipperError> {
    let scaled = scale
        .checked_scale(value)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    Ok(scaled as f32)
}
