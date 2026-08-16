// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:310-358, 7400-7448` and
// `GCodeWriter.cpp` retract, wipe, and spiral-lift motion.

#[cfg(test)]
mod tests;
use super::{EmitState, arc, format::offset as format_offset, format_axis, format_extrusion};

pub(super) fn retract_and_lift(output: &mut Vec<u8>, target: arc::Point, state: &mut EmitState) {
    retract_and_wipe(output, state);
    append_lift(output, target, state);
    state.retracted = true;
}

pub(super) fn retract_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.retracted {
        return;
    }
    retract_and_wipe(output, state);
    append_eager_lift(output, state);
    state.retracted = true;
}

fn retract_and_wipe(output: &mut Vec<u8>, state: &mut EmitState) {
    let wipe_moves = wipe_moves(state);
    let wiped_distance = wipe_moves.iter().map(|(_, length)| length).sum::<f64>();
    let requested_before = state.options.retraction_length * state.options.retract_before_wipe;
    let remaining = state.options.retraction_length - requested_before;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    let during =
        (state.options.retraction_feedrate / 60.0 * wiped_distance / wipe_speed).min(remaining);
    let before = state.options.retraction_length - during;
    if before > f64::EPSILON {
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(-before),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
    }
    if !wipe_moves.is_empty() && during > f64::EPSILON {
        output.extend_from_slice(b"; WIPE_START\n");
        if before > f64::EPSILON && state.options.role_based_wipe_speed {
            output.extend_from_slice(
                format!("G1 F{}\n", format_axis(state.extrusion_feedrate)).as_bytes(),
            );
        }
        let mut remaining = during;
        for (point, length) in wipe_moves {
            let retraction = (during * length / wiped_distance).min(remaining);
            output.extend_from_slice(
                format!(
                    "G1 X{} Y{} E{}\n",
                    format_axis(point.x),
                    format_axis(point.y),
                    format_extrusion(-retraction)
                )
                .as_bytes(),
            );
            state.x = point.x;
            state.y = point.y;
            remaining -= retraction;
        }
        output.extend_from_slice(b"; WIPE_END\n");
    }
}

fn wipe_moves(state: &EmitState) -> Vec<(arc::Point, f64)> {
    if !state.options.wipe || state.options.wipe_distance <= 0.0 {
        return Vec::new();
    }
    let mut output = Vec::new();
    let mut remaining = state.options.wipe_distance;
    let mut current = arc::Point {
        x: state.x,
        y: state.y,
    };
    for pair in state.wipe_path.windows(2).rev() {
        let end = pair[0];
        let length = distance(current, end);
        if length <= f64::EPSILON {
            current = end;
            continue;
        }
        let used = remaining.min(length);
        let target = if used < length {
            let ratio = used / length;
            arc::Point {
                x: current.x + (end.x - current.x) * ratio,
                y: current.y + (end.y - current.y) * ratio,
            }
        } else {
            end
        };
        output.push((target, used));
        remaining -= used;
        if remaining <= f64::EPSILON {
            break;
        }
        current = end;
    }
    output
}

fn append_lift(output: &mut Vec<u8>, target: arc::Point, state: &mut EmitState) {
    if state.options.z_hop <= 0.0 {
        return;
    }
    let raised_z = state.layer_z + state.options.z_hop;
    let dx = target.x - state.x;
    let dy = target.y - state.y;
    let travel_distance = dx.hypot(dy);
    if state.options.spiral_lift
        && state.options.travel_slope_radians > 0.0
        && travel_distance > f64::EPSILON
    {
        let radius = state.options.z_hop
            / (std::f64::consts::TAU * state.options.travel_slope_radians.atan());
        let i = -dy / travel_distance * radius;
        let j = dx / travel_distance * radius;
        output.extend_from_slice(b"G17\n");
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J{} P1  F{}\n",
                format_extrusion(raised_z),
                format_offset(i),
                format_offset(j),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_extrusion(raised_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
    }
    state.lifted = true;
}

fn append_eager_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.z_hop <= 0.0 {
        return;
    }
    let raised_z = state.layer_z + state.options.z_hop;
    if state.options.spiral_lift && state.options.travel_slope_radians > 0.0 {
        let radius = state.options.z_hop
            / (std::f64::consts::TAU * state.options.travel_slope_radians.atan());
        output.extend_from_slice(b"G17\n");
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J0 P1  F{}\n",
                format_extrusion(raised_z),
                format_offset(radius),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_extrusion(raised_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
    }
    state.lifted = true;
}

pub(super) fn inside_internal_surfaces(
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
    start: arc::Point,
    end: arc::Point,
    scale: crate::geometry::CoordinateScale,
    offset: (f64, f64),
) -> bool {
    use crate::project_slice::region_slices::RegionSurfaceKind;

    let point = |point: arc::Point| {
        crate::geometry::Point::new(
            scale.checked_scale(point.x - offset.0).unwrap(),
            scale.checked_scale(point.y - offset.1).unwrap(),
        )
    };
    let travel = crate::geometry::Polyline::new(vec![point(start), point(end)]);
    surfaces.iter().any(|surface| {
        let (kind, region, ..) = surface.as_parts();
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid
        ) && crate::geometry::open_polyline_inside_expolygon(&travel, region).unwrap()
    })
}

fn distance(left: arc::Point, right: arc::Point) -> f64 {
    (left.x - right.x).hypot(left.y - right.y)
}
