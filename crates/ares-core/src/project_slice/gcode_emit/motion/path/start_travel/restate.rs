//! The post-travel z re-statement (`GCode.cpp:6381-6386`) plus the
//! retracted-path descent of an eager-lifted nozzle.

use crate::project_slice::gcode_emit::motion::{
    EmitState, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
    travel,
};

pub(super) struct RestateContext {
    pub(super) slope_start_z: Option<f64>,
    pub(super) never_positioned: bool,
    pub(super) first_position: bool,
    pub(super) travel_set_layer_z: bool,
    pub(super) eager_lifted_travel: bool,
    pub(super) unclear_position_travel: bool,
}

pub(super) fn restate(output: &mut Vec<u8>, state: &mut EmitState, context: RestateContext) {
    let RestateContext {
        slope_start_z,
        never_positioned,
        first_position,
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    } = context;
    let needs_z_restate = never_positioned && (eager_lifted_travel || unclear_position_travel);
    if needs_z_restate && !travel_set_layer_z {
        let z_feedrate = travel::lift_z_feedrate(state);
        let z_comment = state
            .options
            .gcode_comments
            .then_some(" ; ensure Z matches planned layer height")
            .unwrap_or("");
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{z_comment}\n",
                format_z(state.layer_z),
                format_axis(z_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = z_feedrate;
    }
    if state.retracted {
        if first_position
            && state.options.z_hop > 0.0
            && !state.lifted
            && travel::lift_is_allowed_at(state, state.layer_z)
        {
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_z(state.layer_z + state.options.z_hop),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.lifted = true;
            state.lifted_amount = state.options.z_hop;
        }
        if state.lifted && !travel_set_layer_z {
            let z = slope_start_z.unwrap_or(state.layer_z);
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!("G1 Z{} F{}\n", format_z(z), format_axis(z_feedrate)).as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.scarf_z = slope_start_z;
        }
        // `Extruder::unretract()` restores the TRACKED `m_retracted` plus
        // `m_restart_extra` — not the configured retraction length — so a
        // nozzle pre-retracted by start-gcode assignments restores exactly
        // what the template retracted.
        let unretract = extrusion::coordinate(
            state,
            state.retracted_amount + state.options.retract_restart_extra,
        );
        // `Extruder::unretract()` zeroes `m_retracted` after the extrude;
        // `coordinate` only accumulates it for negative deltas.
        state.retracted_amount = 0.0;
        // `GCodeWriter::unretract` emits the G1 only for a non-zero dE
        // (`GCodeProcessor`-side is_zero guard, `GCodeWriter.cpp:1063`):
        // a zero-length retraction (retraction_length=0 with wipe)
        // deretracts silently.
        if unretract.abs() > 0.0 {
            let comment = state
                .options
                .gcode_comments
                .then_some(" ;  ; unretract")
                .unwrap_or("");
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
        state.retracted = false;
        state.lifted = false;
        state.lifted_amount = 0.0;
        state.template_lifted = false;
    }
}
