//! The post-travel z re-statement (`GCode.cpp:6381-6386`) plus the
//! retracted-path descent of an eager-lifted nozzle.

use super::super::travel_emit;
use crate::project_slice::gcode_emit::motion::{
    EmitState, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
    travel,
};

pub(super) struct RestateContext {
    pub(super) slope_start_z: Option<f64>,
    pub(super) never_positioned: bool,
    pub(super) travel_set_layer_z: bool,
    pub(super) eager_lifted_travel: bool,
    pub(super) unclear_position_travel: bool,
}

pub(super) fn restate(output: &mut Vec<u8>, state: &mut EmitState, context: RestateContext) {
    let RestateContext {
        slope_start_z,
        never_positioned,
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    } = context;
    // `GCode.cpp:6396-6400`: for a non-contoured, non-sloped path, any
    // drift between the writer Z and the nominal layer Z (e.g. a
    // role-change hook assigning `position[2]`) is restored with a
    // forced `travel_to_z(m_nominal_z)` — which also clears the
    // outstanding lift (`m_lifted = 0`, `GCodeWriter.cpp:824`).
    let current_z = state.writer_z.unwrap_or(state.layer_z);
    // The reset also clears `m_lifted`; when a retraction is still pending
    // the lifted descend below lands at the same Z with the upstream
    // marker — suppress the double reset only for that combined case. A
    // lift WITHOUT retraction (timelapse `G1 Z`) has no later descend, so
    // the reset must run to restore `m_nominal_z` (`GCode.cpp:6396-6400`).
    let suppressed = state.retracted && (state.lifted_amount > 0.0 || state.template_lifted);
    if slope_start_z.is_none()
        && !suppressed
        && travel_emit::quantize_axis(current_z) != travel_emit::quantize_axis(state.layer_z)
    {
        let z_feedrate = travel::lift_z_feedrate(state);
        let reset_comment = if state.options.gcode_comments {
            " ; reset Z after contouring"
        } else {
            ""
        };
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{reset_comment}\n",
                format_z(state.layer_z),
                format_axis(z_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = z_feedrate;
        state.record_z(state.layer_z);
        // `travel_to_z` cancels `m_lifted` (`GCodeWriter.cpp:824`) — a
        // template-assigned lift is equally cleared since the writer Z is
        // authoritative again.
        state.lifted = false;
        state.lifted_amount = 0.0;
        state.template_lifted = false;
    }
    let needs_z_restate = never_positioned && (eager_lifted_travel || unclear_position_travel);
    if needs_z_restate && !travel_set_layer_z {
        let z_feedrate = travel::lift_z_feedrate(state);
        let z_comment = if state.options.gcode_comments {
            " ; ensure Z matches planned layer height"
        } else {
            ""
        };
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{z_comment}\n",
                format_z(state.layer_z),
                format_axis(z_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = z_feedrate;
        state.record_z(state.layer_z);
    }
    // Upstream calls `unretract()` unconditionally before every
    // extrusion (`GCode.cpp:6407-6408`), and `unretract` runs `unlift()`
    // independently of the E restoration (`GCode.hpp:255`,
    // `GCodeWriter.cpp:1084-1092`) — a template that cleared
    // `e_retracted` after the lift was scheduled must still descend.
    {
        if state.lifted_amount > 0.0 && !travel_set_layer_z {
            let z = slope_start_z.unwrap_or(state.layer_z);
            let z_feedrate = travel::lift_z_feedrate(state);
            // `_extrude` re-states Z: on the print's first extrusion
            // (`_last_pos_undefined`) the comment is "ensure Z matches planned
            // layer height" (`GCode.cpp:6382`); otherwise the post-contour
            // reset uses "reset Z after contouring" (`GCode.cpp:6399`).
            let reset_comment = if state.options.gcode_comments {
                if never_positioned {
                    " ; ensure Z matches planned layer height"
                } else {
                    " ; reset Z after contouring"
                }
            } else {
                ""
            };
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{reset_comment}\n",
                    format_z(z),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.record_z(z);
            state.scarf_z = slope_start_z;
        }
        // `Extruder::unretract()` restores the TRACKED `m_retracted`
        // plus `m_restart_extra`; the is_zero emission gate tests the
        // DELTA (`GCodeWriter.cpp:1063-1076`, `Extruder.cpp:98-103`) —
        // `coordinate` returns the absolute counter, which can land on
        // zero while the delta is real.
        let delta = state.retracted_amount + state.restart_extra;
        let unretract = extrusion::coordinate(state, delta);
        // `Extruder::unretract()` zeroes `m_retracted`/`m_restart_extra`
        // after the extrude (`Extruder.cpp:92-104`).
        state.retracted_amount = 0.0;
        state.restart_extra = 0.0;
        // A zero-length retraction (retraction_length=0 with wipe)
        // deretracts silently.
        if delta.abs() >= 1e-6 {
            if state.options.use_firmware_retraction {
                // `GCodeWriter::unretract` with `use_firmware_retraction`:
                // the bare `G11 ; unretract` (or `G23`) plus `reset_e()`
                // (`GCodeWriter.cpp:1066-1070`).
                output.extend_from_slice(
                    if state.options.gcode_flavor == crate::GCodeFlavor::Machinekit {
                        b"G23 ; unretract\n"
                    } else {
                        b"G11 ; unretract\n"
                    },
                );
                // `reset_e` after the marker is zero-counter-gated and
                // flavor-excluded (`GCodeWriter.cpp:480-499,1064-1070`).
                travel::reset_e(output, state);
            } else {
                let comment = if state.options.gcode_comments {
                    " ;  ; unretract"
                } else {
                    ""
                };
                output.extend_from_slice(
                    format!(
                        "G1 E{} F{}{comment}\n",
                        format_extrusion(unretract),
                        format_axis(state.options.deretraction_feedrate)
                    )
                    .as_bytes(),
                );
                state.current_feedrate = state.options.deretraction_feedrate;
            }
        }
        state.retracted = false;
        state.lifted = false;
        state.lifted_amount = 0.0;
        state.template_lifted = false;
        // `unlift()` also clears the deferred-lift slot
        // (`GCodeWriter.cpp:1084-1092` `m_to_lift = 0`) — a pending hop
        // must not survive into a later unrelated travel when the
        // approach path itself was skipped by a template position
        // write-back.
        state.pending_lift = None;
    }
}
