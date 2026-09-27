//! The non-wiping retraction core (`GCodeWriter::_retract`,
//! `GCodeWriter.cpp:1022-1054`): plain `G1 E-…` at the retraction
//! feedrate, or the firmware `G10`/`G22` marker. `m_restart_extra` is
//! assigned on every call — even a no-op (`Extruder.cpp:77`).

use super::super::extrusion;
use super::super::format::{axis as format_axis, extrusion as format_extrusion};
use super::super::state::EmitState;

pub(super) fn plain_retract(output: &mut Vec<u8>, state: &mut EmitState) {
    // `Extruder::retract` assigns `m_restart_extra` on every call, even
    // when no E moves (`Extruder.cpp:66,77`).
    state.restart_extra = state.options.retract_restart_extra;
    if state.options.use_firmware_retraction {
        firmware_retract(output, state);
        return;
    }
    let remaining = (state.options.retraction_length - state.retracted_amount).max(0.0);
    if remaining <= 0.0 {
        return;
    }
    // `coordinate` always accumulates `m_retracted`; `_retract` emits
    // only when `!is_zero(dE)` (`GCodeWriter.cpp:1035`, `libslic3r.h:294`).
    let retract = extrusion::coordinate(state, -remaining);
    if remaining >= 1e-6 {
        let comment = state
            .options
            .gcode_comments
            .then_some(" ; retract")
            .unwrap_or("");
        output.extend_from_slice(
            format!(
                "G1 E{} F{}{comment}\n",
                format_extrusion(retract),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
        if !state.options.use_relative_e_distances {
            output.extend_from_slice(b"G92 E0\n");
            state.e_position = 0.0;
        }
        state.current_feedrate = state.options.retraction_feedrate;
    }
}

/// `GCodeWriter::_retract` with `use_firmware_retraction`: the length is
/// a fake non-zero 1 and the output is the bare `G10 ; retract` (or `G22`
/// for Machinekit) with a hardcoded comment — no E word, no F word
/// (`GCodeWriter.cpp:1022-1044`).
pub(super) fn firmware_retract(output: &mut Vec<u8>, state: &mut EmitState) {
    state.restart_extra = state.options.retract_restart_extra;
    // Counters advance on every call; the `G10`/`G22` line emits only
    // when `!is_zero(dE)` (`GCodeWriter.cpp:1035`, `libslic3r.h:294`).
    let remaining = (1.0 - state.retracted_amount).max(0.0);
    if remaining >= 1e-6 {
        output.extend_from_slice(
            if state.options.gcode_flavor == crate::GCodeFlavor::Machinekit {
                b"G22 ; retract\n"
            } else {
                b"G10 ; retract\n"
            },
        );
    }
    state.retracted_amount = 1.0;
    state.retracted = true;
}
