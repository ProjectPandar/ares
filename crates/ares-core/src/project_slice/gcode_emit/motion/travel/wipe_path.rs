//! Wipe path construction (`Wipe::wipe`, `GCode.cpp:440-500`) — the
//! stored path → scaled int points → clip_end → segment/dE split.

use super::super::state::EmitState;
use super::arc;

pub(super) struct WipePath {
    pub(super) segments: Vec<(arc::Point, f64)>,
    pub(super) retraction_distance: f64,
    pub(super) distribution_distance: f64,
}

impl Default for WipePath {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            retraction_distance: 0.0,
            distribution_distance: 0.0,
        }
    }
}

pub(super) fn wipe_moves(state: &EmitState) -> WipePath {
    if !state.options.wipe || state.options.wipe_distance <= 0.0 {
        return WipePath::default();
    }
    let start = state.wipe_start.unwrap_or(arc::Point {
        x: state.x,
        y: state.y,
    });
    let mut points = Vec::with_capacity(state.wipe_path.len());
    // Upstream substitutes `gcodegen.last_pos()` — the LOCAL scaled int
    // stored verbatim at the last emitted point (`GCode.cpp:449`), not a
    // rescale of the mm position. Use the cached scaled int directly.
    points.push(if state.last_position_defined {
        state.last_scaled_position
    } else {
        scaled_position(start, state)
    });
    // Upstream `Wipe::path.points` are already scaled ints (the path
    // lives in scaled space); our mm round-trip must recover exactly
    // those ints — round-to-nearest, since `int * 1e-6 / 1e-6` can land
    // one ulp below the integer.
    let mut tail = state.wipe_path.iter().skip(1).map(|&point| {
        (
            (point.x / state.scale_factor).round() as i64,
            (point.y / state.scale_factor).round() as i64,
        )
    });
    // `Polyline::append(begin, end)` skips only a shared junction point.
    if let Some(point) = tail.next()
        && point != points[0]
    {
        points.push(point);
    }
    points.extend(tail);
    let total_length = points
        .windows(2)
        .map(|segment| scaled_distance(segment[0], segment[1]))
        .sum::<f64>();
    // `scale_(x)` is plain division (`libslic3r.h:93`); assigned to
    // `double wipe_dist` upstream so no int truncation applies.
    let configured_distance = state.options.wipe_distance / state.scale_factor;
    // GCode.cpp:342-350 caps the budget at the physical extruder's distance.
    let budget_distance = state.options.retraction_during_wipe_distance / state.scale_factor;
    let retraction_distance = total_length.min(budget_distance) * state.scale_factor;
    let mut clip = total_length - configured_distance;
    // Upstream `Polyline::clip_end` math: squared comparison, sqrt divisor,
    // truncating cast (`Polyline.cpp:52-72`); the loop guard keeps the last
    // point (`Polyline.cpp:56-59` returns when the polyline empties).
    while clip > 0.0 && points.len() > 1 {
        let last = points.pop().unwrap();
        let previous = *points.last().unwrap();
        let vx = (previous.0 - last.0) as f64;
        let vy = (previous.1 - last.1) as f64;
        let lsqr = vx * vx + vy * vy;
        if lsqr > clip * clip {
            let factor = clip / lsqr.sqrt();
            points.push((
                (last.0 as f64 + vx * factor) as i64,
                (last.1 as f64 + vy * factor) as i64,
            ));
            break;
        }
        clip -= lsqr.sqrt();
    }
    let segments = points
        .windows(2)
        .map(|segment| {
            let length = scaled_distance(segment[0], segment[1]);
            (unscaled_position(segment[1], state), length)
        })
        .collect::<Vec<_>>();
    // Upstream `Wipe::wipe` (`GCode.cpp:456-462`): `wipe_dist` stays the
    // CONFIGURED scaled value when the clipped path is at least as long
    // (`if (wipe_path.length() < wipe_dist) wipe_dist = …`); the dE
    // denominator is the exact config value, not the clipped float sum.
    let clipped_distance = segments.iter().map(|(_, length)| length).sum::<f64>();
    let distribution_distance = if clipped_distance < configured_distance {
        // `GCode.cpp:463`: EPSILON is in the already-scaled denominator.
        clipped_distance.max(1e-4)
    } else {
        configured_distance
    };
    WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    }
}

fn scaled_distance(left: (i64, i64), right: (i64, i64)) -> f64 {
    let dx = (left.0 - right.0) as f64;
    let dy = (left.1 - right.1) as f64;
    (dx * dx + dy * dy).sqrt()
}

pub(in crate::project_slice::gcode_emit::motion) fn scaled_position(
    point: arc::Point,
    state: &EmitState,
) -> (i64, i64) {
    // mm→scaled recovery of an upstream-scaled-int position —
    // round-to-nearest (see `wipe_path` points above).
    (
        // `gcode_to_point` evaluation order: `(point - m_origin) +
        // extruder_offset` (`GCode.cpp:8138-8144`) — keep the same
        // association so the round-trip lands on the same double.
        ((point.x - state.origin.0 + state.extruder_offset.0) / state.scale_factor).round() as i64,
        ((point.y - state.origin.1 + state.extruder_offset.1) / state.scale_factor).round() as i64,
    )
}

fn unscaled_position(point: (i64, i64), state: &EmitState) -> arc::Point {
    arc::Point {
        x: point.0 as f64 * state.scale_factor + state.origin.0 - state.extruder_offset.0,
        y: point.1 as f64 * state.scale_factor + state.origin.1 - state.extruder_offset.1,
    }
}
