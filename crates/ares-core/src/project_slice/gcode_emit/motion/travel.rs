// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:310-358, 7400-7448` and
// `GCodeWriter.cpp` retract, wipe, and spiral-lift motion.

#[cfg(test)]
mod tests;
use super::{
    EmitState, LiftMode, arc, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion, offset as format_offset},
};

pub(super) fn retract_and_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe(output, state);
    schedule_lift(state, false);
    state.retracted = true;
}

pub(in crate::project_slice::gcode_emit) fn retract_for_print_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if !state.retracted {
        retract_and_wipe(output, state);
        state.retracted = true;
    }
    state.pending_layer_retract = false;
}

pub(super) fn retract_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.retracted {
        return;
    }
    retract_and_wipe(output, state);
    append_eager_lift(output, state);
    state.retracted = true;
}

/// Non-BBL printers defer the layer-change retraction to after the next
/// layer marker block (`GCode.cpp` change_layer: the wipe lands between
/// the rendered `before_layer_change_gcode` and the cooling fan marker,
/// the lift after it).
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_wipe(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.pending_layer_retract && !state.retracted {
        retract_and_wipe(output, state);
    }
}

pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_lift(
    _output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.pending_layer_retract && !state.retracted {
        schedule_lift(state, true);
        state.retracted = true;
    }
    state.pending_layer_retract = false;
}

fn schedule_lift(state: &mut EmitState, layer_change: bool) {
    if state.options.z_hop <= 0.0 || !lift_height_is_allowed(state) || !lift_is_enforced(state) {
        return;
    }
    state.pending_lift = Some(
        if state.options.spiral_lift || state.options.auto_lift && layer_change {
            LiftMode::Spiral
        } else if state.options.auto_lift {
            LiftMode::Slope
        } else {
            LiftMode::Normal
        },
    );
}

fn lift_height_is_allowed(state: &EmitState) -> bool {
    const EPSILON: f64 = 1.0e-4;

    state.layer_z >= state.options.retract_lift_above - EPSILON
        && (state.options.retract_lift_below == 0.0
            || state.layer_z <= state.options.retract_lift_below + EPSILON)
}

fn lift_is_enforced(state: &EmitState) -> bool {
    use crate::RetractLiftEnforce;

    let bottom = state.layer_index == 0;
    let top = matches!(state.last_feature, Some("Top surface" | "Ironing"));
    match state.options.retract_lift_enforce {
        RetractLiftEnforce::AllSurfaces => true,
        RetractLiftEnforce::TopOnly => top,
        RetractLiftEnforce::BottomOnly => bottom,
        RetractLiftEnforce::TopAndBottom => top || bottom,
    }
}

fn retract_and_wipe(output: &mut Vec<u8>, state: &mut EmitState) {
    let WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    } = wipe_moves(state);
    let requested_before = state.options.retraction_length * state.options.retract_before_wipe;
    let remaining = state.options.retraction_length - requested_before;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    let during = (state.options.retraction_feedrate / 60.0 * retraction_distance / wipe_speed)
        .min(remaining);
    let before = state.options.retraction_length - during;
    if before > f64::EPSILON {
        let retract = extrusion::coordinate(state, -before);
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(retract),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.retraction_feedrate;
    }
    if !segments.is_empty() && during > f64::EPSILON {
        let wipe_start = if state.tags.is_bbl() {
            "; WIPE_START\n"
        } else {
            ";WIPE_START\n"
        };
        output.extend_from_slice(wipe_start.as_bytes());
        output.extend_from_slice(
            format!("G1 F{};_WIPE\n", format_axis(wipe_speed * 60.0)).as_bytes(),
        );
        state.current_feedrate = wipe_speed * 60.0;
        for (point, segment_length) in segments {
            let retraction = during * (segment_length / distribution_distance);
            let retract = extrusion::coordinate(state, -retraction);
            output.extend_from_slice(
                format!(
                    "G1 X{} Y{} E{}\n",
                    format_axis(point.x),
                    format_axis(point.y),
                    format_extrusion(retract)
                )
                .as_bytes(),
            );
            state.x = point.x;
            state.y = point.y;
            state.last_scaled_position = Some((
                ((point.x - state.offset.0) / state.scale_factor).round() as i64,
                ((point.y - state.offset.1) / state.scale_factor).round() as i64,
            ));
            state.wipe_start = Some(point);
        }
        let wipe_end = if state.tags.is_bbl() {
            "; WIPE_END\n"
        } else {
            ";WIPE_END\n"
        };
        output.extend_from_slice(wipe_end.as_bytes());
    }
    if !state.options.use_relative_e_distances && state.e_position.abs() > f64::EPSILON {
        output.extend_from_slice(b"G92 E0\n");
        state.e_position = 0.0;
    }
}

#[derive(Default)]
struct WipePath {
    segments: Vec<(arc::Point, f64)>,
    retraction_distance: f64,
    distribution_distance: f64,
}

fn wipe_moves(state: &EmitState) -> WipePath {
    if !state.options.wipe || state.options.wipe_distance <= 0.0 {
        return WipePath::default();
    }
    let start = state.wipe_start.unwrap_or(arc::Point {
        x: state.x,
        y: state.y,
    });
    let mut points = Vec::with_capacity(state.wipe_path.len());
    points.push(scaled_position(start, state));
    points.extend(
        state
            .wipe_path
            .iter()
            .skip(1)
            .map(|&point| scaled_position(point, state)),
    );
    let total_length = points
        .windows(2)
        .map(|segment| scaled_distance(segment[0], segment[1]))
        .sum::<f64>();
    let configured_distance = state.options.wipe_distance / state.scale_factor;
    let retraction_distance = total_length.min(configured_distance) * state.scale_factor;
    let mut clip = total_length - configured_distance;
    while clip > 0.0 {
        let last = points.pop().unwrap();
        let previous = *points.last().unwrap();
        let dx = (previous.0 - last.0) as f64;
        let dy = (previous.1 - last.1) as f64;
        let length = scaled_distance(last, previous);
        if length > clip {
            points.push((
                (last.0 as f64 + dx * (clip / length)) as i64,
                (last.1 as f64 + dy * (clip / length)) as i64,
            ));
            break;
        }
        clip -= length;
    }
    let segments = points
        .windows(2)
        .filter_map(|segment| {
            let length = scaled_distance(segment[0], segment[1]);
            (length > f64::EPSILON).then(|| (unscaled_position(segment[1], state), length))
        })
        .collect::<Vec<_>>();
    let distribution_distance = segments
        .iter()
        .map(|(_, length)| length)
        .sum::<f64>()
        .min(configured_distance);
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

fn scaled_position(point: arc::Point, state: &EmitState) -> (i64, i64) {
    (
        ((point.x - state.offset.0) / state.scale_factor).round() as i64,
        ((point.y - state.offset.1) / state.scale_factor).round() as i64,
    )
}

fn unscaled_position(point: (i64, i64), state: &EmitState) -> arc::Point {
    arc::Point {
        x: point.0 as f64 * state.scale_factor + state.offset.0,
        y: point.1 as f64 * state.scale_factor + state.offset.1,
    }
}

pub(super) fn emit_pending_lift(
    output: &mut Vec<u8>,
    target: arc::Point,
    state: &mut EmitState,
) -> bool {
    let Some(mode) = state.pending_lift.take() else {
        return false;
    };
    let raised_z = state.layer_z + state.options.z_hop;
    let dx = target.x - state.x;
    let dy = target.y - state.y;
    let travel_distance = dx.hypot(dy);
    let emitted = match mode {
        LiftMode::Normal => {
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_extrusion(raised_z),
                    format_axis(state.travel_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = state.travel_feedrate;
            true
        }
        LiftMode::Spiral => {
            let slope = state.options.travel_slope_radians;
            if slope > 0.0 && travel_distance > f64::EPSILON {
                let radius = state.options.z_hop / (std::f64::consts::TAU * slope.atan());
                let i = -dy / travel_distance * radius;
                let j = dx / travel_distance * radius;
                append_spiral_lift(output, state, raised_z, i, j);
                state.current_feedrate = state.travel_feedrate;
                true
            } else {
                false
            }
        }
        LiftMode::Slope => {
            let slope = state.options.travel_slope_radians;
            if travel_distance > f64::EPSILON && state.options.z_hop.atan2(travel_distance) < slope
            {
                let slope_distance = state.options.z_hop / slope.tan();
                let x = state.x + dx * slope_distance / travel_distance;
                let y = state.y + dy * slope_distance / travel_distance;
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{} F{}\n",
                        format_axis(x),
                        format_axis(y),
                        format_extrusion(raised_z),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                state.current_feedrate = state.travel_feedrate;
                true
            } else {
                false
            }
        }
    };
    state.lifted |= emitted;
    emitted
}

fn append_eager_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.z_hop <= 0.0 || !lift_height_is_allowed(state) {
        return;
    }
    let raised_z = state.layer_z + state.options.z_hop;
    if (state.options.spiral_lift || state.options.auto_lift)
        && state.options.travel_slope_radians > 0.0
    {
        let radius = state.options.z_hop
            / (std::f64::consts::TAU * state.options.travel_slope_radians.atan());
        append_spiral_lift(output, state, raised_z, radius, 0.0);
        state.current_feedrate = state.travel_feedrate;
    } else {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_extrusion(raised_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.travel_feedrate;
    }
    state.lifted = true;
}

fn append_spiral_lift(output: &mut Vec<u8>, state: &EmitState, raised_z: f64, i: f64, j: f64) {
    if state.options.enable_arc_fitting {
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
        return;
    }

    output.extend_from_slice(format!("G1 F{}\n", format_axis(state.travel_feedrate)).as_bytes());
    let resolution = state.options.arc_fitting_tolerance;
    let segments = (8.0 * (0.01 / resolution)).round().clamp(4.0, 16.0) as usize;
    let center_x = state.x + i;
    let center_y = state.y + j;
    let radius = i.hypot(j);
    let start_angle = (state.y - center_y).atan2(state.x - center_x);
    for index in 1..segments {
        let progress = index as f64 / segments as f64;
        let angle = start_angle + std::f64::consts::TAU * progress;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        let z = state.layer_z + (raised_z - state.layer_z) * progress;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{}\n",
                super::super::format_processor_float(x),
                super::super::format_processor_float(y),
                super::super::format_processor_float(z)
            )
            .as_bytes(),
        );
    }
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{}\n",
            super::super::format_processor_float(state.x),
            super::super::format_processor_float(state.y),
            super::super::format_processor_float(raised_z)
        )
        .as_bytes(),
    );
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
