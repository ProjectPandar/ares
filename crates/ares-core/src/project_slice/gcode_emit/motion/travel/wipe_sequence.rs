//! The retract-and-wipe sequence (`GCode.cpp:7659-7674`, `GCodeWriter.cpp:980-1054`):
//! pre-wipe fraction, distributed during-wipe retraction, the wipe path
//! itself, the post-wipe top-up and the trailing `reset_e`.

use super::super::extrusion;
use super::super::format::{axis as format_axis, extrusion as format_extrusion};
use super::super::state::EmitState;
use super::effective_retraction_length;
use super::retract_core::firmware_retract;
use super::wipe_path::{WipePath, wipe_moves};

/// Upstream's `retract()` executes its wipe + `reset_e` phases whenever
/// invoked — even when the E delta is zero — provided an eligible wipe
/// path is stored (`GCode.cpp:7659-7663`).
pub(super) fn wipe_eligible(state: &EmitState) -> bool {
    state.options.wipe && !state.wipe_path.is_empty() && state.options.wipe_distance > 1e-4
}

/// `retract()` runs when it has something to do: a remaining retract
/// distance or an eligible stored wipe path (`GCode.cpp:7659-7670`).
pub(super) fn retract_sequence_due(state: &EmitState) -> bool {
    state.extruder.retracted_amount < effective_retraction_length(state) || wipe_eligible(state)
}

pub(in crate::project_slice::gcode_emit::motion) fn retract_and_wipe(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    retract_and_wipe_with(output, state, true);
}

/// The already-retracted qualifying travel: `retract()` still runs the
/// wipe + `reset_e` side effects (`GCode.cpp:7659-7674`) even though its
/// E delta is zero.
pub(in crate::project_slice::gcode_emit) fn wipe_only_retract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    retract_and_wipe_with(output, state, true);
}

/// The Hilbert-suppressed tail (`GCode.cpp:7667-7672`) skips the final
/// `m_writer.retract()` — `m_restart_extra` keeps the before-wipe
/// fraction assigned by the surviving call
/// (`GCodeWriter.cpp:1006-1011`: `retract_before_wipe *
/// retract_restart_extra`).
pub(in crate::project_slice::gcode_emit) fn retract_and_wipe_suppressed_tail(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    retract_and_wipe_impl(output, state, true, true, false);
}

pub(super) fn retract_and_wipe_with(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    cooling_marker: bool,
) {
    retract_and_wipe_impl(output, state, cooling_marker, false, false);
}

pub(super) fn retract_and_wipe_impl(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    cooling_marker: bool,
    suppressed_tail: bool,
    toolchange: bool,
) {
    let (retraction_length, restart_extra, retract_comment) = if toolchange {
        (
            state.options.retract_length_toolchange,
            state.options.retract_restart_extra_toolchange,
            " ; retract for toolchange",
        )
    } else {
        (
            state.options.retraction_length,
            state.options.retract_restart_extra,
            " ; retract",
        )
    };
    let has_wipe = wipe_eligible(state);
    // `Extruder::retract` assigns `m_restart_extra` on every call
    // (`Extruder.cpp:66,77`): the before-wipe call assigns
    // `retract_before_wipe * extra`, the post-wipe `retract()` call
    // reassigns the full `extra` (`GCodeWriter.cpp:1004-1011`).
    if has_wipe {
        state.extruder.restart_extra = state.options.retract_before_wipe * restart_extra;
    } else if !suppressed_tail {
        state.extruder.restart_extra = restart_extra;
    }
    let WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    } = wipe_moves(state);
    let requested_before = retraction_length * state.options.retract_before_wipe;
    let remaining = retraction_length - requested_before;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    // `GCode.cpp:350-358`: the during-wipe amount is
    // `retraction_speed * unscale_(wipe_path_length) / wipe_speed` (the
    // effective, unrounded mm/s). Upstream caps at `remaining` only when
    // `during - remaining > EPSILON`; within the EPSILON window the
    // larger `during` value is emitted as-is.
    let during_max = state.options.retraction_during_wipe_speed * retraction_distance / wipe_speed;
    // Upstream EPSILON is `1e-4` (`libslic3r.h:52`). When the during
    // amount is itself below EPSILON upstream returns `(fL, 0)` — the
    // whole pre-wipe fraction retracts BEFORE the wipe and the wipe
    // carries no retraction (`GCode.cpp:350-353`). Otherwise `during >
    // remaining` within the EPSILON window emits as-is (:352-357).
    // `GCode.cpp:330,337`: the full-length and remaining-length early
    // returns precede the during-wipe budget calculation.
    let during = if retraction_length <= 1e-4 || remaining <= 1e-4 || during_max <= 1e-4 {
        0.0
    } else if during_max - remaining > 1e-4 {
        remaining
    } else {
        during_max
    };
    // `before` = `retraction_before_wipe + (remaining - during)` =
    // `retraction_length - during` (`GCode.cpp:358` adds the leftover
    // difference into the pre-wipe amount). Without an eligible wipe
    // path upstream never calls `retract(true, beforeWipe)` — the single
    // `retract()` emits only the REMAINING distance
    // (`Extruder.cpp:75-84`), so a partially-retracted nozzle (e.g. a
    // template `e_retracted` assignment) tops up rather than restarting
    // the full length.
    let before = if !has_wipe {
        (retraction_length - state.extruder.retracted_amount).max(0.0)
    } else if during <= 0.0 {
        // `GCodeWriter.cpp:1004-1011` consumes the `(0, 0)` return via
        // its configured before-wipe fraction, as for `(fL, 0)` here.
        // `(fL, 0)` — no leftover difference is folded into the
        // pre-wipe amount (`GCode.cpp:351-352`).
        (requested_before - state.extruder.retracted_amount).max(0.0)
    } else {
        // The before-wipe call emits `max(0, before - m_retracted)`
        // (`Extruder.cpp:77-84`); the post-wipe `retract()` tops the
        // remainder off to the full length.
        // `GCodeWriter.cpp:1004-1011`: only overrides above EPSILON
        // select the explicit length; otherwise use the configured fraction.
        let override_length = retraction_length - during;
        let length = if override_length > 1e-4 {
            override_length
        } else {
            requested_before
        };
        (length - state.extruder.retracted_amount).max(0.0)
    };
    if state.options.use_firmware_retraction && (has_wipe || !suppressed_tail) {
        firmware_retract(output, state);
        state.extruder.retracted = true;
    } else if has_wipe || !suppressed_tail {
        // `extrusion::coordinate` accumulates `retracted_amount`
        // (`Extruder.cpp:77-84` semantics live in the shared counter);
        // `_retract` emits only when `!is_zero(dE)` (`GCodeWriter.cpp:
        // 1035`, `libslic3r.h:294` — 1e-6).
        let retract = extrusion::coordinate(state, -before);
        if before >= 1e-6 {
            emit_retract_e(output, state, retract, retract_comment);
        }
    }
    if has_wipe {
        let wipe_start = if state.tags.is_bbl() {
            "; WIPE_START\n"
        } else {
            ";WIPE_START\n"
        };
        output.extend_from_slice(wipe_start.as_bytes());
        let speed_line = if cooling_marker {
            format!("G1 F{};_WIPE\n", format_axis(wipe_speed * 60.0))
        } else {
            format!("G1 F{}\n", format_axis(wipe_speed * 60.0))
        };
        output.extend_from_slice(speed_line.as_bytes());
        state.current_feedrate = wipe_speed * 60.0;
        for (point, segment_length) in segments {
            let retraction = during * (segment_length / distribution_distance);
            // Orca's `extrude_to_xy` omits the E word when the wipe carries
            // no retraction (`retract_before_wipe` = 100% leaves nothing to
            // distribute during the wipe).
            let line = if retraction > f64::EPSILON {
                let retract = extrusion::coordinate(state, -retraction);
                let comment = gcode_comment(state, " ; wipe and retract");
                format!(
                    "G1 X{} Y{} E{}{comment}\n",
                    format_axis(point.x),
                    format_axis(point.y),
                    format_extrusion(retract)
                )
            } else {
                let comment = gcode_comment(state, " ; wipe and retract");
                format!(
                    "G1 X{} Y{}{comment}\n",
                    format_axis(point.x),
                    format_axis(point.y)
                )
            };
            output.extend_from_slice(line.as_bytes());
            state.x = point.x;
            state.y = point.y;
            // Recover the scaled-int position the mm value came from
            // (upstream wipe moves hold ints; mm→int round-trips are
            // exact under round-to-nearest).
            // `(point - m_origin) + extruder_offset` order
            // (`GCode.cpp:8138-8144`).
            state.last_scaled_position = (
                ((point.x - state.origin.0 + state.extruder_offset.0) / state.scale_factor).round()
                    as i64,
                ((point.y - state.origin.1 + state.extruder_offset.1) / state.scale_factor).round()
                    as i64,
            );
            state.last_position_defined = true;
            state.wipe_start = Some(point);
        }
        let wipe_end = if state.tags.is_bbl() {
            "; WIPE_END\n"
        } else {
            ";WIPE_END\n"
        };
        output.extend_from_slice(wipe_end.as_bytes());
        // `Wipe::wipe` resets the stored path so a subsequent retract
        // does not re-wipe the same path (`GCode.cpp:497`).
        state.wipe_path.clear();
        // Upstream always calls the final `retract()` after the wipe —
        // it tops the extruder off to the full length whenever the
        // wipe carried less than the remaining distance
        // (`GCode.cpp:7664-7670`, `Extruder.cpp:75-84`). For the
        // suppressed tail the final call is skipped entirely.
        if !suppressed_tail {
            state.extruder.restart_extra = restart_extra;
        }
        if !suppressed_tail && state.options.use_firmware_retraction {
            firmware_retract(output, state);
        } else if !suppressed_tail {
            let topup = (retraction_length - state.extruder.retracted_amount).max(0.0);
            // `Extruder::retract` always accumulates `m_retracted`;
            // `_retract` emits only when `!is_zero(dE)` — `1e-6`, not
            // machine eps (`GCodeWriter.cpp:1035`, `libslic3r.h:294`).
            // `coordinate` applies the counter update for both cases.
            let retract = extrusion::coordinate(state, -topup);
            if topup >= 1e-6 {
                emit_retract_e(output, state, retract, retract_comment);
            }
        }
    }
    reset_e(output, state);
}

/// `GCodeWriter::_retract` comment gating: the feature comment is
/// appended only under `gcode_comments` (`GCodeWriter.cpp:1035-1042`).
pub(in crate::project_slice::gcode_emit) fn gcode_comment<'a>(
    state: &EmitState,
    comment: &'a str,
) -> &'a str {
    if state.options.gcode_comments {
        comment
    } else {
        ""
    }
}

/// Emits the `G1 E<retract> F<feed>` retraction line and latches the
/// feedrate (`GCodeWriter.cpp:1035-1042`). The caller decides whether
/// the delta clears `is_zero` (1e-6) before invoking.
fn emit_retract_e(output: &mut Vec<u8>, state: &mut EmitState, d_e: f64, comment: &str) {
    let comment = gcode_comment(state, comment);
    output.extend_from_slice(
        format!(
            "G1 E{} F{}{comment}\n",
            format_extrusion(d_e),
            format_axis(state.options.retraction_feedrate)
        )
        .as_bytes(),
    );
    state.current_feedrate = state.options.retraction_feedrate;
}

/// `GCodeWriter::reset_e` (`GCodeWriter.cpp:480-499`): emits
/// `G92 E0` (+ ` ; reset extrusion distance` under gcode_comments)
/// only when `!is_zero(E)` — 1e-6 — and never for
/// Mach3/MakerWare/Sailfish.
pub(in crate::project_slice::gcode_emit) fn reset_e(output: &mut Vec<u8>, state: &mut EmitState) {
    if !state.options.use_relative_e_distances
        && !matches!(
            state.options.gcode_flavor,
            crate::GCodeFlavor::Mach3
                | crate::GCodeFlavor::MakerWare
                | crate::GCodeFlavor::Sailfish
        )
        && state.extruder.e_position.abs() >= 1e-6
    {
        let comment = gcode_comment(state, " ; reset extrusion distance");
        output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
        state.extruder.e_position = 0.0;
    }
}
