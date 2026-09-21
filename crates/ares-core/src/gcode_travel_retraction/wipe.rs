//! `WipePath` (`GCode.hpp:59-71`): the accumulated last extrusion path
//! for wiping — fed per extruded path at `GCode.cpp:5980-5990`, reset
//! after wiping at `:496`. Includes the scaled-length and clip-to-length
//! helpers upstream delegates to `MultiPoint::length` /
//! `Polyline::clip_end`.

use crate::Point2;
use crate::gcode_lift::distance;

/// Scaled-coordinate space of `libslic3r` points (1e6 units per mm).
pub(super) const SCALE: f64 = 1_000_000.0;

/// The accumulated last extrusion path for wiping.
#[derive(Debug, Default)]
pub(super) struct WipePath {
    pub(super) points: Vec<Point2>,
    pub(super) feedrate: f64,
}

/// `Line::length()`: `sqrt(dx² + dy²)` over scaled integer coordinates.
pub(super) fn scaled_length(start: Point2, end: Point2) -> f64 {
    let dx = (end.x() - start.x()) * SCALE;
    let dy = (end.y() - start.y()) * SCALE;
    (dx * dx + dy * dy).sqrt()
}

impl WipePath {
    pub(super) fn observe_print_move(&mut self, start: Point2, end: Point2, feedrate: f64) {
        if distance(start, end) <= f64::EPSILON {
            return;
        }
        match self.points.last() {
            Some(&last) if last == start => {
                // `GCode.cpp:5984-5988`: don't save duplicated points.
                if self.points.last() != Some(&end) {
                    self.points.push(end);
                }
                self.feedrate = feedrate;
            }
            _ => {
                // A new path begins (travel gap or fresh start): replace.
                self.points = vec![start, end];
                self.feedrate = feedrate;
            }
        }
    }

    /// `Wipe::wipe` `:450-453`: `[last_pos] + path.points[1..]` over the
    /// REVERSED path (the wipe walks backward from the current position;
    /// `m_wipe.path` stores the reversed emission order).
    pub(super) fn with_current_position(&self, current: Point2) -> Vec<Point2> {
        let mut points: Vec<Point2> = self.points.iter().rev().copied().collect();
        if let Some(first) = points.first_mut() {
            *first = current;
        }
        points
    }

    pub(super) fn length(points: &[Point2]) -> f64 {
        // `MultiPoint::length` sums `Line::length()` — `sqrt(dx²+dy²)` over
        // the SCALED INTEGER points (`Polyline.cpp`), not the compensated
        // `hypot` over millimetres.
        points
            .windows(2)
            .map(|pair| scaled_length(pair[0], pair[1]))
            .sum()
    }

    /// `Polyline::clip_end(len - wipe_dist)` (`:456`): walk from the END,
    /// interpolate the boundary point onto the INTEGER lattice (the C++
    /// `cast<coord_t>()` truncates toward zero) — the truncated endpoint
    /// shifts the segment ratio and flips 5th-decimal E knife edges.
    pub(super) fn clip_to_length(points: &[Point2], keep: f64) -> Vec<Point2> {
        let total = Self::length(points);
        if total <= keep || points.len() < 2 {
            return points.to_vec();
        }
        // `clip_end(total - keep)` removes from the tail; equivalently keep
        // the leading `keep` millimetres, interpolating the final point with
        // the upstream backward formula truncated onto the lattice.
        let mut kept: Vec<Point2> = points.to_vec();
        let mut distance = total - keep;
        while distance > 0.0 {
            let Some(last_point) = kept.pop() else {
                return Vec::new();
            };
            if kept.is_empty() {
                return Vec::new();
            }
            let previous = *kept.last().expect("checked non-empty");
            let v = (
                (previous.x() - last_point.x()) * SCALE,
                (previous.y() - last_point.y()) * SCALE,
            );
            let lsqr = v.0 * v.0 + v.1 * v.1;
            if lsqr > distance * distance {
                let factor = distance / lsqr.sqrt();
                let x = last_point.x() * SCALE + v.0 * factor;
                let y = last_point.y() * SCALE + v.1 * factor;
                kept.push(Point2::new(
                    x.trunc() as i64 as f64 / SCALE,
                    y.trunc() as i64 as f64 / SCALE,
                ));
                break;
            }
            distance -= lsqr.sqrt();
        }
        kept
    }
}
