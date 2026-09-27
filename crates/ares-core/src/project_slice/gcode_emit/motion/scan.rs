//! The second-layer first-layer-inspection scan (`GCode.cpp:4780-4790`):
//! BBL printers with `scan_first_layer` retract around
//! `M976 S1 P1` + `M400 P100`.

use super::extrusion;
use super::format;
use super::state::EmitState;

pub(super) fn scan_first_layer_block(output: &mut Vec<u8>, state: &mut EmitState) {
    // `this->retract()` (`GCode.cpp:4784`) assigns `m_restart_extra` even
    // when the extruder is already retracted (`Extruder.cpp:77`) and tops
    // up only the REMAINING length — a template `e_retracted` write-back
    // (`e_retracted[0]=0.25` under length 1) retracts another 0.75.
    state.restart_extra = state.options.retract_restart_extra;
    let remaining = (state.options.retraction_length - state.retracted_amount).max(0.0);
    // `coordinate` accumulates `m_retracted` unconditionally; `_retract`
    // emits only when `!is_zero(dE)` (`GCodeWriter.cpp:1035`,
    // `libslic3r.h:294` — 1e-6), so a sub-threshold top-up is silent.
    let retract = extrusion::coordinate(state, -remaining);
    if remaining > 0.0 {
        state.retracted = true;
    }
    if remaining >= 1e-6 {
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
        state.current_feedrate = state.options.retraction_feedrate;
    }
    // `retract()` returns `gcode + reset_e()` — the reset runs on every
    // call (`GCode.cpp:7674`), carrying the upstream comment + flavor
    // exclusions (`GCodeWriter.cpp:480-499`).
    super::travel::reset_e(output, state);
    output.extend_from_slice(b"M976 S1 P1 ; scan model before printing 2nd layer\nM400 P100\n");
    // `unretract()` (`GCode.cpp:4789`) is `unlift() + E restore`
    // (`GCode.hpp:255`): the descent emits FIRST — `G1 Z… F…` plus
    // ` ; restore layer Z` under `gcode_comments`
    // (`GCodeWriter.cpp:1084-1092`) — gated on `m_lifted > 0` (a
    // template-adopted `lifted` flag with zero amount emits nothing), and
    // the deferred hop is always cleared (`m_to_lift = 0`).
    if state.lifted_amount > 0.0 {
        let live = state.writer_z.unwrap_or(state.layer_z);
        let dest = live - state.lifted_amount;
        // `_travel_to_z` uses `travel_speed_z` with the configured
        // first-layer/normal travel fallback (`GCodeWriter.cpp:831-846`)
        // — the CURRENT layer's configured values, not the stale cached
        // `state.travel_feedrate`.
        let z_feedrate = if state.options.z_travel_feedrate > 0.0 {
            state.options.z_travel_feedrate
        } else if state.layer_index == 0 {
            state.options.first_layer_travel_feedrate
        } else {
            state.options.travel_feedrate
        };
        let comment = state
            .options
            .gcode_comments
            .then_some(" ; restore layer Z")
            .unwrap_or("");
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{comment}\n",
                format::z(dest),
                format::axis(z_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = z_feedrate;
        state.record_z(dest);
    }
    state.lifted = false;
    state.lifted_amount = 0.0;
    state.template_lifted = false;
    state.pending_lift = None;
    // `Extruder::unretract()` (`Extruder.cpp:92-104`) restores
    // `m_retracted + m_restart_extra` and clears both unconditionally;
    // the writer gates emission on the delta (`GCodeWriter.cpp:1063`).
    let delta = state.retracted_amount + state.restart_extra;
    let unretract = extrusion::coordinate(state, delta);
    state.retracted_amount = 0.0;
    state.restart_extra = 0.0;
    if delta.abs() >= 1e-6 {
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
