//! The first-travel approach branch chain (`GCodeWriter.cpp:685-806`):
//! spiral-vase, template-lifted, lifted, layer-change and same-layer
//! travel shapes.

use super::super::super::{retraction, travel_emit};
use super::dispatch;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LiftMode,
    format::{axis as format_axis, z as format_z},
    travel,
};

pub(super) struct MoveContext<'a> {
    pub travel_x: f64,
    pub travel_y: f64,
    pub first_travel_comment: &'a str,
    pub first_position: bool,
    pub layer_change_travel: bool,
    /// `state.pending_lift.is_some()` captured before consumption — the
    //   deferred hop that applies to THIS travel.
    pub lifted_for_travel: bool,
    /// `emit_pending`'s raised destination (when a pending lift ran).
    pub pending_target: Option<f64>,
    pub first_travel_lift: Option<LiftMode>,
    pub slope_start_z: Option<f64>,
    pub target_z: f64,
    /// `route.len()` — detour first legs lift to the nominal layer z.
    pub route_len: usize,
}

/// Outcome folded back into `emit_needs_travel`'s triple.
pub(super) struct MoveOutcome {
    pub travel_set_layer_z: bool,
    pub eager_lifted_travel: bool,
    pub unclear_position_travel: bool,
}

#[allow(clippy::too_many_lines)]
pub(super) fn emit_approach_move(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    ctx: MoveContext<'_>,
) -> MoveOutcome {
    let MoveContext {
        travel_x,
        travel_y,
        first_travel_comment,
        first_position,
        layer_change_travel,
        lifted_for_travel,
        pending_target,
        first_travel_lift,
        slope_start_z,
        target_z,
        route_len,
    } = ctx;
    // The first DETOUR waypoint's destination z is the nominal layer z
    // (`m_nominal_z`, `GCode.cpp:7489-7492`) — the scarf `z` ratio only
    // reaches the route's final leg (`waypoints`). `target_z` already
    // resolves to the scarf start on the direct path (`start_travel.rs`).
    let leg_z = if route_len > 1 {
        state.layer_z
    } else {
        slope_start_z.unwrap_or(target_z)
    };
    let mut travel_set_layer_z = false;
    let mut eager_lifted_travel = false;
    let mut unclear_position_travel = false;
    if state.spiral_vase
        && state.lifted
        && !first_position
        && (layer_change_travel
            || (state.layer_z + state.options.z_hop) - state.options.z_hop > state.layer_z)
    {
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            state.layer_z,
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if state.template_lifted && state.lifted && !first_position {
        // Pure XY while still lifted: upstream's no-Z-change branch calls
        // `travel_to_xy(to_2d(point))` WITHOUT the comment
        // (`GCodeWriter.cpp:771-780`).
        travel_emit::xy_with_comment(output, travel_x, travel_y, state.travel_feedrate, "");
        state.template_lifted = false;
    } else if state.lifted {
        eager_lifted_travel = true;
        if !lifted_for_travel {
            travel_emit::xy_with_comment(output, travel_x, travel_y, state.travel_feedrate, "");
        } else {
            // The lifted nozzle travels at the raised destination —
            // when the pending-lift preliminary move didn't fit (short
            // slope), upstream still targets `dest_point(2) = m_to_lift
            // + m_pos(2)` (`GCodeWriter.cpp:705-710`), which
            // `emit_pending` computed but did not emit as a Z move.
            let raised = pending_target.unwrap_or_else(|| state.writer_z.unwrap_or(state.layer_z));
            if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz_with_comment(
                    output,
                    state,
                    travel_x,
                    travel_y,
                    raised,
                    state.travel_feedrate,
                    first_travel_comment,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}{first_travel_comment}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(raised)
                    )
                    .as_bytes(),
                );
                state.record_z(raised);
            }
        }
    } else if layer_change_travel && state.retracted {
        // With a hop actually deferred, the earlier `state.lifted`
        // branch emitted the ramp and the combined layer+hop travel;
        // with no hop in play (the enforce gate blocked the
        // change-layer hop), `m_need_change_layer_lift_z`
        // (`GCode.cpp:7479-7482`) forces the plain combined xyz move.
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            leg_z,
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if first_position && (first_travel_lift.is_some() || state.pending_lift.is_some()) {
        // `m_to_lift` consumption is independent of the retracted flag
        // (`GCodeWriter.cpp:694-710`) — a deferred hop (scheduled by a
        // qualifying retraction, including a Hilbert-suppressed one or
        // the retract path's own deferral) always emits on the first
        // travel. No scheduled lift → no hop (`GCode.cpp:7530-7533`).
        let lift = first_travel_lift.or_else(|| state.pending_lift.take());
        dispatch::emit_retracted_first_travel(
            output,
            state,
            travel_x,
            travel_y,
            first_travel_comment,
            lift,
            // `is_current_position_clear` decides the XY+Z-restate vs
            // combined-XYZ shape (`GCodeWriter.cpp:748-777`) — a
            // machine-start `position` write-back seeds the cursor
            // (`m_last_pos`) but leaves the position unclear
            // (`GCode.cpp:3139-3140`), so track `state.positioned`, not
            // `never_positioned`.
            !state.positioned,
            // The upstream lift compares against this leg's travel
            // destination: the NOMINAL layer z on a detour first leg,
            // the scarf slope start on the direct leg
            // (`GCode.cpp:7481-7498`).
            if route_len > 1 { state.layer_z } else { leg_z },
        );
    } else if layer_change_travel {
        // The sloped split only applies when a lift actually exists
        // (`GCodeWriter.cpp:725-757`: slope/spiral moves come from the
        // `m_to_lift` block; with no lift the plain combined XYZ path
        // emits the destination directly).
        let has_lift = state.lifted || state.pending_lift.is_some();
        if has_lift
            && state.options.z_hop > 0.0
            && retraction::uses_sloped_lift(state.options.z_hop_type)
        {
            travel_emit::xy_with_comment(
                output,
                travel_x,
                travel_y,
                state.travel_feedrate,
                first_travel_comment,
            );
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!("G1 Z{} F{}\n", format_z(leg_z), format_axis(z_feedrate)).as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.record_z(leg_z);
        } else {
            // `travel_to_xyz`'s force-z/`will_move_z` else-branch emits
            // the combined move at `config.travel_speed` UNCONDITIONALLY
            // (`GCodeWriter.cpp:783-806`) — the first-layer travel speed
            // does not apply to a layer-change approach that carries Z.
            travel_emit::xyz_with_comment(
                output,
                state,
                travel_x,
                travel_y,
                leg_z,
                state.options.travel_feedrate,
                first_travel_comment,
            );
        }
        travel_set_layer_z = true;
    } else if let Some(z) = pending_target {
        // A consumed pending lift whose raised target clamped to the
        // layer z (no hop left) still carries a Z word: upstream's
        // `travel_to_xyz` built `dest_point` from `m_to_lift + m_pos`
        // and emits the combined `G1 X Y Z` even when the result isn't
        // "lifted" (`GCodeWriter.cpp:701-768`). Without this the XY-only
        // fallback would leave the writer at the stale Z.
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            z,
            state.travel_feedrate,
            first_travel_comment,
        );
        state.record_z(z);
        travel_set_layer_z = z == leg_z;
    } else if slope_start_z.is_some() {
        // The first leg's destination z equals the live writer z on a
        // same-z detour (or a clamped approach): upstream's
        // `travel_to_xyz` then takes the XY-only, COMMENTLESS
        // `travel_to_xy(to_2d(point))` branch (`GCodeWriter.cpp:771-780`).
        let live_z = state.writer_z.unwrap_or(state.layer_z);
        // Upstream takes the XY-only branch when `std::abs(delta(2)) <
        // EPSILON` — STRICTLY below 1e-4 (`GCodeWriter.cpp:927`,
        // `libslic3r.h:52`).
        if (leg_z - live_z).abs() < 1e-4 {
            travel_emit::xy_with_comment(output, travel_x, travel_y, state.travel_feedrate, "");
        } else {
            travel_emit::xyz_with_comment(
                output,
                state,
                travel_x,
                travel_y,
                leg_z,
                state.travel_feedrate,
                first_travel_comment,
            );
        }
        travel_set_layer_z = true;
    } else {
        // Same `travel_to_xyz` else-branch: the unclear-position split
        // emits the XY leg at `config.travel_speed` without the
        // first-layer override (`GCodeWriter.cpp:793-798`).
        let xy_feedrate = if first_position {
            state.options.travel_feedrate
        } else {
            state.travel_feedrate
        };
        // The no-Z-change `travel_to_xy(to_2d(point))` branch drops the
        // comment for ordinary same-Z XY travels (`GCodeWriter.cpp:780`);
        // the unclear-position split (first travel) DOES emit it on both
        // legs (`GCodeWriter.cpp:754-763`).
        travel_emit::xy_with_comment(
            output,
            travel_x,
            travel_y,
            xy_feedrate,
            if first_position {
                first_travel_comment
            } else {
                ""
            },
        );
        // The print's first travel from an unknown position always
        // splits: the unclear-position branch of `travel_to_xyz`
        // (`GCodeWriter.cpp:754+`) emits XY then `_travel_to_z`
        // separately — `force_z` only merges Z for CLEAR-position
        // travels.
        if first_position {
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{first_travel_comment}\n",
                    format_z(leg_z),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.record_z(leg_z);
            unclear_position_travel = true;
        }
    }
    MoveOutcome {
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    }
}
