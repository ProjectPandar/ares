//! The retract-and-wipe sequence (`GCode.cpp:7659-7674`, `GCodeWriter.cpp:980-1054`):
//! pre-wipe fraction, distributed during-wipe retraction, the wipe path
//! itself, the post-wipe top-up and the trailing `reset_e`.

use super::super::extrusion;
use super::super::format::{axis as format_axis, extrusion as format_extrusion};
use super::super::state::EmitState;
use super::effective_retraction_length;
use super::retract_core::firmware_retract;
use super::wipe_path::wipe_moves;

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
    let wipe = wipe_moves(state);
    let retraction_distance = wipe.retraction_distance;
    let (override_length, during) =
        wipe_retraction_lengths(state, retraction_length, retraction_distance);
    let requested_before = retraction_length * state.options.retract_before_wipe;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    let before = if !has_wipe {
        (retraction_length - state.extruder.retracted_amount).max(0.0)
    } else {
        // GCodeWriter::_retract dispatches only overrides above EPSILON.
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
        super::wipe_execute::emit(output, state, wipe, (during, wipe_speed, cooling_marker));
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
    reset_e(output, state, false);
}

// GCode.cpp:538-602 (345efec64c7b): reserve both full-length products
// before calculating the during-wipe budget. The ordinary retract supplies after.
fn wipe_retraction_lengths(state: &EmitState, length: f64, distance: f64) -> (f64, f64) {
    if length <= 1e-4 {
        return (0.0, 0.0);
    }
    let mut before = length * state.options.retract_before_wipe;
    let after = length * state.options.retract_after_wipe;
    let remaining = length - (before + after);
    if remaining <= 1e-4 {
        return (before, 0.0);
    }
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    let during = state.options.retraction_during_wipe_speed * distance / wipe_speed;
    if during <= 1e-4 {
        return (before, 0.0);
    }
    if during - remaining > 1e-4 {
        return (before, remaining);
    }
    before += remaining - during;
    (before, during)
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

/// `GCodeWriter::reset_e(force)` (`GCodeWriter.cpp:480-502`): excluded
/// flavors do nothing; otherwise only `is_zero(E) && !force` suppresses
/// the reset. Clear the counter in both E modes, emitting the existing
/// reset line and optional comment only for absolute E.
pub(in crate::project_slice::gcode_emit) fn reset_e(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    force: bool,
) {
    if matches!(
        state.options.gcode_flavor,
        crate::GCodeFlavor::Mach3 | crate::GCodeFlavor::MakerWare | crate::GCodeFlavor::Sailfish
    ) || (state.extruder.e_position.abs() < 1e-6 && !force)
    {
        return;
    }
    state.extruder.e_position = 0.0;
    if !state.options.use_relative_e_distances {
        let comment = gcode_comment(state, " ; reset extrusion distance");
        output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
    }
}
