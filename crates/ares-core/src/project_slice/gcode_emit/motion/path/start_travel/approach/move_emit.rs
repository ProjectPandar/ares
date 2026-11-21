//! The first-travel approach branch chain (`GCodeWriter.cpp:685-806`):
//! spiral-vase, template-lifted, lifted, layer-change and same-layer
//! travel shapes.

use super::super::super::{retraction, travel_emit};
use super::dispatch;
use super::travel_will_move_z;
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
    /// A pending lift strictly above EPSILON, captured before consumption.
    /// This is the deferred hop that applies to THIS travel.
    pub lifted_for_travel: bool,
    /// `emit_pending`'s raised destination (when a pending lift ran).
    pub pending_target: Option<f64>,
    pub first_travel_lift: Option<LiftMode>,
    pub slope_start_z: Option<f64>,
    pub leg_z: f64,
    pub type2_entry: bool,
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
        leg_z,
        type2_entry,
    } = ctx;
    let mut travel_set_layer_z = false;
    let mut eager_lifted_travel = false;
    let mut unclear_position_travel = false;
    // Actual-Z-move branch: `else { m_lifted = 0; }`
    // (`GCodeWriter.cpp:783-785`) — when the destination sits OUTSIDE
    // the lifted dead-band the travel cancels the lift entirely.
    // A consumed pending lift (`m_to_lift` branch, `GCodeWriter.cpp:
    // 692-768`) must NOT consult `will_move_z` again: upstream emits the
    // combined `xy_z_move` at the RAISED `dest_point(2)` unconditionally
    // and keeps `m_lifted` for the later descend — checking `leg_z`
    // against the dead-band here would cancel the lift and merge the
    // descent into the travel (Artillery M1 Pro `G1 X..Y..Z.16`).
    // `will_move_z` true while lifted: upstream cancels the lift and
    // emits the combined `G1 X Y Z{dest}` (`GCodeWriter.cpp:782-806`)
    // — including the floating-point edge where `pos - m_lifted`
    // lands one ulp above the nominal z (0.6 - 0.4 > 0.2), which
    // legitimately merges a lift+descend pair that an epsilon-based
    // check would leave split.
    // `travel_to_xyz` bypasses the lifted dead-band when force_z is set
    // (GCodeWriter.cpp:771). This flag also applies to an unclear position
    // after timelapse; the approach-shape flag excludes that case.
    let cancelled_lift = pending_target.is_none()
        && state.lifted_amount > 0.0
        && (state.layer_change_travel_pending || travel_will_move_z(state, leg_z));
    let rebase_lift =
        type2_entry && !cancelled_lift && pending_target.is_none() && state.lifted_amount > 0.0;
    if cancelled_lift {
        state.lifted = false;
        state.lifted_amount = 0.0;
    }
    if state.spiral_vase
        && state.lifted_amount > 0.0
        && !first_position
        && (layer_change_travel
            || (state.layer_z + state.options.z_hop) - state.options.z_hop > state.layer_z)
    {
        travel_emit::xyz_with_comment(
            output,
            state,
            crate::Point3d {
                x: travel_x,
                y: travel_y,
                z: state.layer_z,
            },
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if state.template_lifted
        && state.lifted_amount > 0.0
        && !first_position
        && !travel_will_move_z(state, leg_z)
    {
        // Pure XY while still lifted: upstream's no-Z-change branch calls
        // `travel_to_xy(to_2d(point))` WITHOUT the comment
        // (`GCodeWriter.cpp:771-780`).
        travel_emit::xy_with_comment(
            output,
            state,
            (travel_x, travel_y),
            state.travel_feedrate,
            "",
        );
        state.template_lifted = false;
    } else if state.lifted_amount > 0.0 && (lifted_for_travel || !travel_will_move_z(state, leg_z))
    {
        eager_lifted_travel = true;
        if !lifted_for_travel {
            travel_emit::xy_with_comment(
                output,
                state,
                (travel_x, travel_y),
                state.travel_feedrate,
                "",
            );
        } else {
            // The lifted nozzle travels at the raised destination —
            // when the pending-lift preliminary move didn't fit (short
            // slope), upstream still targets `dest_point(2) = m_to_lift
            // + m_pos(2)` (`GCodeWriter.cpp:705-710`), which
            // `emit_pending` computed but did not emit as a Z move.
            let raised = pending_target.unwrap_or_else(|| state.writer_z.unwrap_or(state.layer_z));
            travel_emit::xyz_with_comment(
                output,
                state,
                crate::Point3d {
                    x: travel_x,
                    y: travel_y,
                    z: raised,
                },
                state.travel_feedrate,
                first_travel_comment,
            );
        }
    } else if layer_change_travel && state.extruder.retracted {
        // With a hop actually deferred, the earlier `state.lifted_amount > 0.0`
        // branch emitted the ramp and the combined layer+hop travel;
        // with no hop in play (the enforce gate blocked the
        // change-layer hop), `m_need_change_layer_lift_z`
        // (`GCode.cpp:7479-7482`) forces the plain combined xyz move.
        travel_emit::xyz_with_comment(
            output,
            state,
            crate::Point3d {
                x: travel_x,
                y: travel_y,
                z: leg_z,
            },
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if cancelled_lift && state.positioned {
        // Clear-position `emit_xyz(point_on_plate)` path: one combined
        // `G1 X Y Z` travel line at the travel feedrate
        // (`GCodeWriter.cpp:799-805`). The unclear-position variant
        // keeps the XY+`_travel_to_z` split via the `else` arm below.
        travel_emit::xyz_with_comment(
            output,
            state,
            crate::Point3d {
                x: travel_x,
                y: travel_y,
                z: leg_z,
            },
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if first_position
        && state.options.z_hop.abs() > travel::EPSILON
        && (first_travel_lift.is_some() || state.pending_lift.is_some())
    {
        // `GCodeWriter.cpp:700-710`: only a deferred hop above EPSILON is
        // consumed, independently of the retracted flag. A subthreshold
        // pending lift takes the ordinary travel branch and remains set.
        let lift = first_travel_lift.or_else(|| state.pending_lift.take());
        dispatch::emit_retracted_first_travel(
            output,
            state,
            dispatch::FirstTravel {
                travel_x,
                travel_y,
                comment: first_travel_comment,
                lift,
                // `is_current_position_clear` decides the XY+Z-restate vs
                // combined-XYZ shape (`GCodeWriter.cpp:748-777`) — a
                // machine-start `position` write-back seeds the cursor
                // (`m_last_pos`) but leaves the position unclear
                // (`GCode.cpp:3139-3140`), so track `state.positioned`, not
                // `never_positioned`.
                never_positioned: !state.positioned,
                // The upstream lift compares against this leg's travel
                // destination: the NOMINAL layer z on a detour first leg,
                // the scarf slope start on the direct leg
                // (`GCode.cpp:7481-7498`).
                dest_z: leg_z,
            },
        );
    } else if layer_change_travel {
        // The sloped split only applies when a lift actually exists
        // (`GCodeWriter.cpp:725-757`: slope/spiral moves come from the
        // `m_to_lift` block; with no lift the plain combined XYZ path
        // emits the destination directly).
        let has_lift = state.lifted_amount > 0.0
            || (state.pending_lift.is_some() && state.options.z_hop.abs() > travel::EPSILON);
        if has_lift
            && state.options.z_hop > 0.0
            && retraction::uses_sloped_lift(state.options.z_hop_type)
        {
            travel_emit::xy_with_comment(
                output,
                state,
                (travel_x, travel_y),
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
                crate::Point3d {
                    x: travel_x,
                    y: travel_y,
                    z: leg_z,
                },
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
            crate::Point3d {
                x: travel_x,
                y: travel_y,
                z,
            },
            state.travel_feedrate,
            first_travel_comment,
        );
        state.record_z(z);
        travel_set_layer_z = z == leg_z;
    } else if slope_start_z.is_some() {
        // GCode.cpp:6371-6394/7480-7482 supplies contour/sloped or nominal Z;
        // GCodeWriter.cpp:771-806 handles their XY/XYZ travel branches.
        // The first leg's destination z equals the live writer z on a
        // same-z detour (or a clamped approach): upstream's
        // `travel_to_xyz` then takes the XY-only, COMMENTLESS
        // `travel_to_xy(to_2d(point))` branch (`GCodeWriter.cpp:771-780`).
        let live_z = state.writer_z.unwrap_or(state.layer_z);
        // Upstream takes the XY-only branch when `std::abs(delta(2)) <
        // EPSILON` — STRICTLY below 1e-4 (`GCodeWriter.cpp:927`,
        // `libslic3r.h:52`).
        if (leg_z - live_z).abs() < travel::EPSILON {
            travel_emit::xy_with_comment(
                output,
                state,
                (travel_x, travel_y),
                state.travel_feedrate,
                "",
            );
        } else if !state.positioned {
            // Unclear position: upstream's `travel_to_xyz` splits into
            // XY (with the travel comment) then a separate `_travel_to_z`
            // re-statement (`GCodeWriter.cpp:792-798, 753-764`).
            travel_emit::xy_with_comment(
                output,
                state,
                (travel_x, travel_y),
                state.travel_feedrate,
                first_travel_comment,
            );
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
        } else {
            travel_emit::xyz_with_comment(
                output,
                state,
                crate::Point3d {
                    x: travel_x,
                    y: travel_y,
                    z: leg_z,
                },
                state.travel_feedrate,
                first_travel_comment,
            );
        }
        travel_set_layer_z = true;
    } else {
        // Same-Z returns before the unclear-position split. Cancelling a lift
        // already selected the actual-Z branch (`GCodeWriter.cpp:1096–1139`).
        let split_position = first_position
            && (cancelled_lift
                || state.writer_z.is_none()
                || state.layer_change_travel_pending
                || travel_will_move_z(state, leg_z));
        let xy_feedrate = if split_position {
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
            state,
            (travel_x, travel_y),
            xy_feedrate,
            if split_position {
                first_travel_comment
            } else {
                ""
            },
        );
        // The first extrusion still synchronizes Z in `restate` after XY-only travel.
        unclear_position_travel = first_position;
        if split_position {
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
        }
    }
    if rebase_lift && !travel_set_layer_z {
        // `travel_to_xyz` rebases its lift on the XY-only branch
        // (GCodeWriter.cpp:1096–1105). Preserve the subtraction order.
        let nominal_z = state.writer_z.unwrap_or(state.layer_z) - state.lifted_amount;
        state.lifted_amount -= leg_z - nominal_z;
        if state.lifted_amount.abs() < travel::EPSILON {
            state.lifted_amount = 0.0;
            state.lifted = false;
        }
    }
    MoveOutcome {
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    }
}
