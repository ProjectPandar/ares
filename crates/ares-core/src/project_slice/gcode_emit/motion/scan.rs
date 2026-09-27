//! The second-layer first-layer-inspection scan (`GCode.cpp:4780-4790`):
//! BBL printers with `scan_first_layer` retract around
//! `M976 S1 P1` + `M400 P100`.

use super::extrusion;
use super::format;
use super::state::EmitState;

pub(super) fn scan_first_layer_block(output: &mut Vec<u8>, state: &mut EmitState) {
    // `this->retract()` (`GCode.cpp:4784`) assigns `m_restart_extra` even
    // when the extruder is already retracted (`Extruder.cpp:77`).
    state.restart_extra = state.options.retract_restart_extra;
    if !state.retracted {
        let length = state.options.retraction_length;
        let remaining = (length - state.retracted_amount).max(0.0);
        if remaining > 0.0 {
            let retract = extrusion::coordinate(state, -remaining);
            let comment = state
                .options
                .gcode_comments
                .then_some(" ; retract")
                .unwrap_or("");
            output.extend_from_slice(
                format!(
                    "G1 E{} F{}{comment}\n",
                    format::extrusion(retract),
                    format::axis(state.options.retraction_feedrate)
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
    output.extend_from_slice(b"M976 S1 P1 ; scan model before printing 2nd layer\nM400 P100\n");
    if state.retracted || state.retracted_amount > 0.0 {
        let unretract = extrusion::coordinate(state, state.retracted_amount + state.restart_extra);
        state.retracted_amount = 0.0;
        state.restart_extra = 0.0;
        if unretract.abs() > 0.0 {
            output.extend_from_slice(
                format!(
                    "G1 E{} F{}\n",
                    format::extrusion(unretract),
                    format::axis(state.options.deretraction_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = state.options.deretraction_feedrate;
        }
        state.retracted = false;
    }
    // `unretract()` (`GCode.cpp:4789`) also invokes `unlift()`
    // (`GCode.hpp:255`), which emits the descent `G1 Z…` while lifted
    // and always clears the deferred hop (`GCodeWriter.cpp:1084-1092` —
    // `m_to_lift = 0`) — even when the extruder carried nothing to
    // restore.
    if state.lifted {
        let live = state.writer_z.unwrap_or(state.layer_z);
        let dest = live - state.lifted_amount;
        let z_feedrate = super::travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!("G1 Z{} F{}\n", format::z(dest), format::axis(z_feedrate)).as_bytes(),
        );
        state.current_feedrate = z_feedrate;
        state.record_z(dest);
    }
    state.lifted = false;
    state.lifted_amount = 0.0;
    state.template_lifted = false;
    state.pending_lift = None;
}
