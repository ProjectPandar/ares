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
        let comment = if state.options.gcode_comments {
            " ; retract"
        } else {
            ""
        };
        output.extend_from_slice(
            format!(
                "G1 E{} F{}{comment}\n",
                format_extrusion(retract),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
        // `reset_e` (`GCodeWriter.cpp:480-499`): skipped for
        // Mach3/MakerWare/Sailfish and a zero counter.
        if !state.options.use_relative_e_distances
            && !matches!(
                state.options.gcode_flavor,
                crate::GCodeFlavor::Mach3
                    | crate::GCodeFlavor::MakerWare
                    | crate::GCodeFlavor::Sailfish
            )
            && state.e_position.abs() >= 1e-6
        {
            let comment = if state.options.gcode_comments {
                " ; reset extrusion distance"
            } else {
                ""
            };
            output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
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
    // `Extruder::retract` ALSO decrements the E counter by the fake
    // amount (`Extruder.cpp:75-84` — `m_E` moves even without an E
    // word), so the unretract bookkeeping balances and `reset_e`'s
    // counter gate sees the dip.
    let remaining = (1.0 - state.retracted_amount).max(0.0);
    let _ = extrusion::coordinate(state, -remaining);
    if remaining >= 1e-6 {
        output.extend_from_slice(
            if state.options.gcode_flavor == crate::GCodeFlavor::Machinekit {
                b"G22 ; retract\n"
            } else {
                b"G10 ; retract\n"
            },
        );
    }
    // `coordinate` already accumulated the fake amount into
    // `retracted_amount` (`Extruder.cpp:75-84` semantics); a
    // template-reported value above 1.0 is preserved.
    state.retracted = true;
}
