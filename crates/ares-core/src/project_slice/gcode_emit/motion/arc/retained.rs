#[cfg(test)]
mod tests;

use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::materialize::{FittedArc, FittedMove},
};

use super::{ArcSegment, Point, Segment, append_line};

pub(in crate::project_slice::gcode_emit::motion) fn from_fitting(
    points: &[Point],
    fitting: &[FittedMove],
    origin: (f64, f64),
    extruder_offset: (f64, f64),
) -> Vec<Segment> {
    let mut segments = Vec::with_capacity(points.len());
    for fitted in fitting {
        if let Some(arc) = fitted.arc {
            segments.push(Segment::Arc(ArcSegment {
                start: points[fitted.start],
                end: points[fitted.end],
                // `GCode.cpp:8124-8128`: add origin before subtracting the
                // extruder offset; folding them changes I/J at rounding ties.
                center: Point {
                    x: arc.center.0 + origin.0 - extruder_offset.0,
                    y: arc.center.1 + origin.1 - extruder_offset.1,
                },
                radius: arc.radius,
                length: arc.length,
                clockwise: arc.clockwise,
            }))
        } else {
            for pair in points[fitted.start..=fitted.end].windows(2) {
                append_line(&mut segments, pair[0], pair[1]);
            }
        }
    }
    segments
}

pub(in crate::project_slice::gcode_emit::motion) fn clip_end(
    points: &mut [(f64, f64)],
    fitting: &mut Vec<FittedMove>,
    scale: CoordinateScale,
) {
    if points.len() < 2 || fitting.is_empty() {
        fitting.clear();
        return;
    }
    let last_segment_start = points.len() - 1;
    fitting.retain(|fitted| fitted.start < last_segment_start);
    let Some(fitted) = fitting.last_mut() else {
        return;
    };
    fitted.end = points.len() - 1;
    let Some(arc) = &mut fitted.arc else {
        return;
    };
    if !point_within_arc(*arc, points[fitted.start], points[fitted.end]) {
        fitted.arc = None;
        return;
    }
    let Some(endpoint) = project_to_circle(*arc, points[fitted.end], scale) else {
        fitted.arc = None;
        return;
    };
    points[fitted.end] = endpoint;
    if !update_arc_length(arc, points[fitted.start], endpoint) {
        fitted.arc = None;
    }
}

fn project_to_circle(
    arc: FittedArc,
    point: (f64, f64),
    scale: CoordinateScale,
) -> Option<(f64, f64)> {
    // `ArcSegment::clip_end` -> `Circle::get_closest_point`
    // (`Circle.hpp:24-28`): upstream works on SCALED ints —
    // `v = (input - center).normalized()`, `end = center + trunc(v * radius)`
    // where `center`/`end_point` are `Point` (int) and `radius` the scaled
    // double length. Compute in the int domain so trunc boundaries match
    // upstream exactly.
    let center_int = (
        (arc.center.0 / scale.factor()).round() as i64,
        (arc.center.1 / scale.factor()).round() as i64,
    );
    let input_int = (
        (point.0 / scale.factor()).round() as i64,
        (point.1 / scale.factor()).round() as i64,
    );
    let vx = (input_int.0 - center_int.0) as f64;
    let vy = (input_int.1 - center_int.1) as f64;
    let norm = vx.hypot(vy);
    if norm <= f64::EPSILON {
        return None;
    }
    // Upstream `radius` is the scaled DOUBLE length — no int trunc.
    let radius_scaled = arc.radius / scale.factor();
    let end_int = (
        center_int.0 + (vx / norm * radius_scaled) as i64,
        center_int.1 + (vy / norm * radius_scaled) as i64,
    );
    Some((scale.unscale(end_int.0), scale.unscale(end_int.1)))
}

fn point_within_arc(arc: FittedArc, start: (f64, f64), point: (f64, f64)) -> bool {
    let start_angle = (start.1 - arc.center.1).atan2(start.0 - arc.center.0);
    let point_angle = (point.1 - arc.center.1).atan2(point.0 - arc.center.0);
    let mut delta = point_angle - start_angle;
    if delta > 0.0 && arc.clockwise {
        delta -= std::f64::consts::TAU;
    } else if delta < 0.0 && !arc.clockwise {
        delta += std::f64::consts::TAU;
    }
    let sweep = arc.length / arc.radius;
    if arc.clockwise {
        delta < 0.0 && delta > -sweep
    } else {
        delta > 0.0 && delta < sweep
    }
}

fn update_arc_length(arc: &mut FittedArc, start: (f64, f64), end: (f64, f64)) -> bool {
    let start_angle = (start.1 - arc.center.1).atan2(start.0 - arc.center.0);
    let end_angle = (end.1 - arc.center.1).atan2(end.0 - arc.center.0);
    let sweep = if arc.clockwise {
        (start_angle - end_angle).rem_euclid(std::f64::consts::TAU)
    } else {
        (end_angle - start_angle).rem_euclid(std::f64::consts::TAU)
    };
    arc.length = arc.radius * sweep;
    arc.length > f64::EPSILON
}
