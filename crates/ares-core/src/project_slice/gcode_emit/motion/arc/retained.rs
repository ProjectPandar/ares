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
    offset: (f64, f64),
) -> Vec<Segment> {
    let mut segments = Vec::with_capacity(points.len());
    for fitted in fitting {
        if let Some(arc) = fitted.arc {
            segments.push(Segment::Arc(ArcSegment {
                start: points[fitted.start],
                end: points[fitted.end],
                center: Point {
                    x: arc.center.0 + offset.0,
                    y: arc.center.1 + offset.1,
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
    let dx = point.0 - arc.center.0;
    let dy = point.1 - arc.center.1;
    let distance = dx.hypot(dy);
    (distance > f64::EPSILON).then(|| {
        let x = scale.checked_scale(dx * arc.radius / distance)?;
        let y = scale.checked_scale(dy * arc.radius / distance)?;
        Some((
            arc.center.0 + scale.unscale(x),
            arc.center.1 + scale.unscale(y),
        ))
    })?
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
