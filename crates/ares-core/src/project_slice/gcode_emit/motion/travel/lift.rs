use super::super::{
    EmitState, LiftMode, arc,
    format::{axis as format_axis, offset as format_offset, z as format_z},
};

#[cfg(test)]
mod tests;

pub(super) fn schedule(state: &mut EmitState, layer_change: bool) {
    // `lazy_lift` reads the live `m_pos(2)` (`GCodeWriter.cpp:633-639`)
    // — template `position[2]` assignments move the eligibility origin.
    schedule_at(state, layer_change, state.writer_z.unwrap_or(state.layer_z));
}

/// Upstream `lazy_lift` gates the hop on the writer's *current* z
/// (`GCodeWriter.cpp:633-639` `retract_lift_above/below` vs `m_pos.z()`);
/// `change_layer` fires its retract before the layer z move, so the
/// deferred layer-change lift must evaluate the gate at the previous
/// layer's z, not the new one.
pub(super) fn schedule_at(state: &mut EmitState, layer_change: bool, writer_z: f64) {
    // `GCode.cpp:7678-7681` — `needs_lift` requires a non-zero retraction
    // length (or firmware retraction / toolchange): with retraction
    // disabled (nil length, e.g. Wanhao), `maybe_zlift` never runs and
    // no hop is ever deferred — including the layer-change retract, which
    // is not a toolchange upstream.
    if state.options.retraction_length <= 0.0 && !state.options.use_firmware_retraction {
        return;
    }
    if state.options.z_hop <= 0.0 || !is_allowed_at(state, writer_z) {
        return;
    }
    // `maybe_zlift` only schedules when `m_lifted == 0 && m_to_lift == 0`
    // (`GCodeWriter.cpp:637`) — a lift already scheduled (e.g. by the
    // layer-change retract before a template cleared `e_retracted`)
    // survives a subsequent partial-retract top-up.
    if state.lifted || state.pending_lift.is_some() {
        return;
    }
    state.pending_lift = Some(mode_for(state, layer_change));
}

/// `lazy_lift(NormalLift)` — the fixed lift type used by scan/
/// timelapse/wrapping retract calls (`GCode.hpp:254` default
/// `LiftType::NormalLift`; `auto_lift_type` is NormalLift outside
/// spiral vase).
pub(in crate::project_slice::gcode_emit::motion) fn schedule_normal(state: &mut EmitState) {
    if state.options.retraction_length <= 0.0 && !state.options.use_firmware_retraction {
        return;
    }
    let writer_z = state.writer_z.unwrap_or(state.layer_z);
    if state.options.z_hop <= 0.0 || !is_allowed_at(state, writer_z) {
        return;
    }
    if state.lifted || state.pending_lift.is_some() {
        return;
    }
    state.pending_lift = Some(LiftMode::Normal);
}

/// The `LiftMode` a retract at this state schedules; shared with the
/// first-travel emission, which may run after a retraction that fired
/// before any lift could be scheduled.
pub(in crate::project_slice::gcode_emit) fn mode_for(
    state: &EmitState,
    layer_change: bool,
) -> LiftMode {
    match state.options.z_hop_type {
        crate::ZHopType::Auto if layer_change => LiftMode::Spiral,
        crate::ZHopType::Auto | crate::ZHopType::Slope => LiftMode::Slope,
        crate::ZHopType::Normal => LiftMode::Normal,
        crate::ZHopType::Spiral => LiftMode::Spiral,
    }
}

fn height_is_allowed_at(state: &EmitState, layer_z: f64) -> bool {
    // Raw comparisons (`GCodeWriter.cpp:633-639` gates on `m_pos.z()` with
    // no tolerance): the writer's z is the full-precision accumulated
    // print z, so a layer exactly at `retract_lift_below` (0.2+0.2+0.2 =
    // 0.6000000000000001 > 0.6) is ABOVE the boundary and must not lift.
    // An epsilon tolerance keeps those layers lifting and desyncs every
    // following travel.
    layer_z >= state.options.retract_lift_above
        && (state.options.retract_lift_below == 0.0 || layer_z <= state.options.retract_lift_below)
}

fn height_is_allowed(state: &EmitState) -> bool {
    height_is_allowed_at(state, state.layer_z)
}

pub(in crate::project_slice::gcode_emit::motion) fn is_allowed_at(
    state: &EmitState,
    writer_z: f64,
) -> bool {
    use crate::RetractLiftEnforce;

    if !height_is_allowed_at(state, writer_z) {
        return false;
    }
    // `GCode.cpp:7691-7694`: the bottom-surface test reads
    // `m_layer_index == 0` — the physical counter (raft layers included).
    let bottom = state.layer_index == 0;
    // `m_last_notgapfill_extrusion_role == erTopSolidInfill || erIroning`
    // (`GCode.cpp:7682`) — the non-gapfill feature tracker.
    let top = matches!(
        state.last_notgapfill_feature,
        Some("Top surface" | "Ironing")
    );
    match state.options.retract_lift_enforce {
        RetractLiftEnforce::AllSurfaces => true,
        RetractLiftEnforce::TopOnly => top,
        RetractLiftEnforce::BottomOnly => bottom,
        RetractLiftEnforce::TopAndBottom => top || bottom,
    }
}

/// Upstream `_travel_to_z` feedrate (`GCodeWriter.cpp:832-842`):
/// `travel_speed_z` when non-zero, else the first-layer or normal travel
/// speed per `m_is_first_layer`. Every standalone `G1 Z` travel carries it.
pub(in crate::project_slice::gcode_emit) fn z_feedrate(state: &EmitState) -> f64 {
    if state.options.z_travel_feedrate > 0.0 {
        state.options.z_travel_feedrate
    } else if state.layer_index == 0 {
        state.options.first_layer_travel_feedrate
    } else {
        state.travel_feedrate
    }
}

pub(in crate::project_slice::gcode_emit::motion) fn emit_pending(
    output: &mut Vec<u8>,
    target: arc::Point,
    dest_z: f64,
    state: &mut EmitState,
) -> Option<f64> {
    let mode = state.pending_lift.take()?;
    // `GCodeWriter.cpp:701-709`: the deferred lift raises from the LIVE
    // writer z (`dest_point(2) = m_to_lift + m_pos(2)`), not the nominal
    // layer z — but only while that raised target actually exceeds the
    // travel's destination z (`m_to_lift + m_pos(2) > point(2)`, where
    // `point` is the ACTUAL travel target — a scarf seam's
    // `slope_start_z` or the plain layer z), otherwise `m_lifted` stays
    // 0 and the travel happens at `dest_z`.
    let live_z = state.writer_z.unwrap_or(state.layer_z);
    let raised_z = if live_z + state.options.z_hop > dest_z {
        live_z + state.options.z_hop
    } else {
        dest_z
    };
    let dx = target.x - state.x;
    let dy = target.y - state.y;
    // `GCodeWriter::travel_to_xyz` measures the XY distance with Eigen
    // `Vec2d::norm()` (`sqrt(dx*dx + dy*dy)`, `GCodeWriter.cpp:718`), not
    // `hypot`; the spiral points land on 6-significant-digit rounding
    // boundaries where the extra `hypot` precision flips the last digit
    // (qVMcEP X91.7173 vs upstream X91.7174).
    let travel_distance = (dx * dx + dy * dy).sqrt();
    // The rise is the actual destination delta — a clamped lift can be
    // taller than `z_hop` (`GCodeWriter.cpp:716-740` uses `delta(2)`).
    let rise = raised_z - state.writer_z.unwrap_or(state.layer_z);
    let emitted = match mode {
        LiftMode::Normal => {
            let feedrate = z_feedrate(state);
            // `GCodeWriter::_travel_to_z` (`GCodeWriter.cpp:841`): the lift
            // carries a trailing ` ; normal lift Z` under `gcode_comments`.
            let comment = state
                .options
                .gcode_comments
                .then_some(" ; normal lift Z")
                .unwrap_or("");
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{comment}\n",
                    format_z(raised_z),
                    format_axis(feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = feedrate;
            true
        }
        LiftMode::Spiral => {
            let slope = state.options.travel_slope_radians;
            if slope > 0.0 && travel_distance > f64::EPSILON && rise > 0.0 {
                let radius = rise / (std::f64::consts::TAU * slope.atan());
                append_spiral(
                    output,
                    state,
                    raised_z,
                    -dy / travel_distance * radius,
                    dx / travel_distance * radius,
                );
                state.current_feedrate = state.travel_feedrate;
                true
            } else {
                false
            }
        }
        LiftMode::Slope => {
            let slope = state.options.travel_slope_radians;
            if travel_distance > f64::EPSILON && rise > 0.0 && rise.atan2(travel_distance) < slope {
                let slope_distance = rise / slope.tan();
                let x = state.x + dx * slope_distance / travel_distance;
                let y = state.y + dy * slope_distance / travel_distance;
                // `_travel_to_z(.., "normal lift Z")` (`GCodeWriter.cpp:747`)
                // trails the slope XYZ move under `gcode_comments`.
                let comment = state
                    .options
                    .gcode_comments
                    .then_some(" ; normal lift Z")
                    .unwrap_or("");
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{} F{}{comment}\n",
                        format_axis(x),
                        format_axis(y),
                        format_z(raised_z),
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
    if emitted {
        // The emitted lift move carries a real Z — `m_pos.z()` advances
        // (`GCodeWriter::travel_to_z` sets `m_pos` on every emit).
        state.record_z(raised_z);
    }
    // Upstream assigns `m_lifted` BEFORE the slope-fit test
    // (`GCodeWriter.cpp:705-710`) — a short slope travel keeps the
    // raised destination + lifted state even without a preliminary
    // move; the caller's final XYZ then carries `raised_z`.
    if raised_z > dest_z {
        state.lifted = true;
        state.lifted_amount = raised_z - dest_z;
    }
    Some(raised_z)
}

pub(super) fn append_eager(output: &mut Vec<u8>, state: &mut EmitState) {
    // Upstream `eager_lift` (`GCodeWriter.cpp:652-681`) gates and targets
    // the CURRENT writer position `m_pos.z()` — a second eager lift stacks
    // on the already-raised z, not the nominal layer z (`0.6` then `1.0`,
    // not `0.6` twice). `writer_z` carries the last emitted Z; before any
    // Z move it falls back to the layer z (the writer position).
    // `needs_lift` (`GCode.cpp:7678-7681,7693-7696`): eager lifts only
    // exist under a real retraction — a retracted flag set by the
    // wrapping insert with zero retraction length must not produce a hop.
    if state.options.retraction_length <= 0.0 && !state.options.use_firmware_retraction {
        return;
    }
    let current_z = state.writer_z.map_or(state.layer_z, f64::from);
    if state.options.z_hop <= 0.0 || !height_is_allowed_at(state, current_z) {
        return;
    }
    let raised_z = current_z + state.options.z_hop;
    // `GCodeWriter::eager_lift`: the spiral form requires a known-clear
    // position; an unknown position (right after the start g-code) falls
    // back to the normal Z-only lift. `process_layer` maps Auto, Spiral AND
    // Slope hop types onto SpiralLift for the layer-chunk retract
    // (`GCode.cpp:4657-4660`), so the BBL eager layer-start lift takes the
    // spiral form for all three.
    if matches!(
        state.options.z_hop_type,
        crate::ZHopType::Spiral | crate::ZHopType::Auto | crate::ZHopType::Slope
    ) && state.options.travel_slope_radians > 0.0
        && state.positioned
    {
        let radius = state.options.z_hop
            / (std::f64::consts::TAU * state.options.travel_slope_radians.atan());
        append_spiral(output, state, raised_z, radius, 0.0);
        state.current_feedrate = state.travel_feedrate;
    } else {
        let feedrate = z_feedrate(state);
        let comment = state
            .options
            .gcode_comments
            .then_some(" ; normal lift Z")
            .unwrap_or("");
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{comment}\n",
                format_z(raised_z),
                format_axis(feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = feedrate;
    }
    state.record_z(raised_z);
    state.lifted = true;
    // `eager_lift` stores the configured hop (`m_lifted = target_lift`,
    // `GCodeWriter.cpp:680`) — the raised-above-nominal bookkeeping
    // belongs to deferred travel consumption.
    state.lifted_amount = state.options.z_hop;
    // `GCodeWriter::eager_lift` zeroes `m_to_lift` (`GCodeWriter.cpp:679`):
    // a hop deferred by an earlier retract is realized by this eager lift,
    // so the following travel sees the nozzle already lifted and emits a
    // pure XY move (`will_move_z` false → `travel_to_xy`).
    state.pending_lift = None;
}

pub(super) fn append_spiral_vase(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.z_hop <= 0.0
        || !(state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
        || !is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z))
    {
        return;
    }
    let feedrate = z_feedrate(state);
    let raised = state.writer_z.unwrap_or(state.layer_z) + state.options.z_hop;
    output.extend_from_slice(
        format!("G1 Z{} F{}\n", format_z(raised), format_axis(feedrate)).as_bytes(),
    );
    state.current_feedrate = feedrate;
    state.record_z(raised);
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
}

fn append_spiral(output: &mut Vec<u8>, state: &mut EmitState, raised_z: f64, i: f64, j: f64) {
    // `GCodeWriter.cpp:861`: the linearized spiral interpolates Z from
    // `z_start = m_pos(2)` — the CURRENT writer z (a stacked eager lift
    // continues upward), not the nominal layer z.
    let z_start = state.writer_z.map_or(state.layer_z, f64::from);
    // `_spiral_travel_to_z` (`GCodeWriter.cpp:881`): under
    // `gcode_comments` the comment is a standalone `;spiral lift Z` line
    // before the feedrate set (arc-fitting uses ` ; XY plane for arc`).
    if state.options.enable_arc_fitting {
        if state.options.gcode_comments {
            output.extend_from_slice(b"G17 ; XY plane for arc\n");
        } else {
            output.extend_from_slice(b"G17\n");
        }
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J{} P1  F{}\n",
                format_z(raised_z),
                format_offset(i),
                format_offset(j),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        state.record_z(raised_z);
        return;
    }

    if state.options.gcode_comments {
        output.extend_from_slice(b";spiral lift Z\n");
    }
    output.extend_from_slice(format!("G1 F{}\n", format_axis(state.travel_feedrate)).as_bytes());
    let segments = (8.0 * (0.01 / state.options.arc_fitting_tolerance))
        .round()
        .clamp(4.0, 16.0) as usize;
    let center_x = state.x + i;
    let center_y = state.y + j;
    // `_spiral_travel_to_z` derives the circle radius from
    // `ij_offset.norm()` (`GCodeWriter.cpp:876`) — `sqrt(i*i + j*j)` with
    // its double rounding, not `hypot`.
    let radius = (i * i + j * j).sqrt();
    let start_angle = (state.y - center_y).atan2(state.x - center_x);
    for index in 1..segments {
        let progress = index as f64 / segments as f64;
        let angle = start_angle + std::f64::consts::TAU * progress;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{}\n",
                super::super::super::format_processor_float(center_x + radius * angle.cos()),
                super::super::super::format_processor_float(center_y + radius * angle.sin()),
                super::super::super::format_processor_float(
                    z_start + (raised_z - z_start) * progress
                )
            )
            .as_bytes(),
        );
    }
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{}\n",
            super::super::super::format_processor_float(state.x),
            super::super::super::format_processor_float(state.y),
            super::super::super::format_processor_float(raised_z)
        )
        .as_bytes(),
    );
    state.record_z(raised_z);
}
