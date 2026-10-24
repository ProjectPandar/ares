//! The second-layer first-layer-inspection scan (`GCode.cpp:4780-4790`):
//! BBL printers with `scan_first_layer` retract around
//! `M976 S1 P1` + `M400 P100`.

use super::extrusion;
use super::format;
use super::state::EmitState;

pub(super) fn scan_first_layer_block(output: &mut Vec<u8>, state: &mut EmitState) {
    // `this->retract()` (`GCode.cpp:4784`) is the full
    // `GCode::retract(false, false)` — wipe path, top-up, `reset_e`,
    // then `lazy_lift(NormalLift, spiral_vase)` (`GCode.cpp:7659-7704`,
    // `GCode.hpp:254`): the DEFAULT lift type is NormalLift — never the
    // configured z_hop type — and spiral vase lifts emit EAGERLY.
    super::travel::wipe_only_retract(output, state);
    if state.spiral_vase {
        // `lazy_lift` guards `m_lifted == 0 && m_to_lift == 0` for BOTH
        // variants (`GCodeWriter.cpp:637`) — an already-lifted nozzle
        // does not rise again.
        if state.lifted_amount <= 0.0
            && state.pending_lift.is_none()
            && state.options.z_hop > 0.0
            && (state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
            && super::travel::lift_is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z))
        {
            // `_travel_to_z` resolves `travel_speed_z`, then the
            // first-layer/normal travel feedrate — configured values,
            // not the stale `state.travel_feedrate` cache
            // (`GCodeWriter.cpp:831-846`; the second layer was already
            // selected at `GCode.cpp:4590`).
            let feedrate = if state.options.z_travel_feedrate > 0.0 {
                state.options.z_travel_feedrate
            } else if state.layer_index == 0 {
                state.options.first_layer_travel_feedrate
            } else {
                state.options.travel_feedrate
            };
            let raised = state.writer_z.unwrap_or(state.layer_z) + state.options.z_hop;
            let comment = if state.options.gcode_comments {
                " ; lift Z"
            } else {
                ""
            };
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{comment}\n",
                    format::z(raised),
                    format::axis(feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = feedrate;
            state.record_z(raised);
            state.lifted = true;
            state.lifted_amount = state.options.z_hop;
        }
    } else {
        super::travel::schedule_normal(state);
    }
    state.extruder.retracted = true;
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
        let comment = if state.options.gcode_comments {
            " ; restore layer Z"
        } else {
            ""
        };
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
    let delta = state.extruder.retracted_amount + state.extruder.restart_extra;
    let unretract = extrusion::coordinate(state, delta);
    state.extruder.retracted_amount = 0.0;
    state.extruder.restart_extra = 0.0;
    if delta.abs() >= 1e-6 {
        if state.options.use_firmware_retraction {
            // `GCodeWriter::unretract` firmware branch — bare `G11`
            // (or `G23` Machinekit) + `reset_e()`
            // (`GCodeWriter.cpp:1064-1070`).
            output.extend_from_slice(
                if state.options.gcode_flavor == crate::GCodeFlavor::Machinekit {
                    b"G23 ; unretract\n"
                } else {
                    b"G11 ; unretract\n"
                },
            );
            // `reset_e()` follows the firmware marker only
            // (`GCodeWriter.cpp:1064-1077`); the ordinary E path keeps
            // the absolute counter.
            super::travel::reset_e(output, state);
        } else {
            let comment = if state.options.gcode_comments {
                " ;  ; unretract"
            } else {
                ""
            };
            output.extend_from_slice(
                format!(
                    "G1 E{} F{}{comment}\n",
                    format::extrusion(unretract),
                    format::axis(state.options.deretraction_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = state.options.deretraction_feedrate;
        }
    }
    state.extruder.retracted = false;
}
