//! First-travel emit dispatch branches — the `travel_to_xyz` lift
//! variants (`GCodeWriter.cpp:685-806`).

use super::super::travel_emit;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LiftMode,
    format::{axis as format_axis, z as format_z},
    travel,
};

/// `state.retracted && first_position && z_hop > 0` branch of
/// `emit_needs_travel`: emit the Z lift, then the approach move whose
/// shape depends on whether the source position is genuinely unknown
/// (`GCodeWriter.cpp:748-777`).
pub(super) fn emit_retracted_first_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    travel_x: f64,
    travel_y: f64,
    first_travel_comment: &str,
    first_travel_lift: Option<LiftMode>,
    never_positioned: bool,
) {
    let mode = first_travel_lift.unwrap_or_else(|| travel::lift_mode_for(state, true));
    if mode == LiftMode::Normal {
        let feedrate = travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z + state.options.z_hop),
                format_axis(feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = feedrate;
        if never_positioned {
            // Genuinely unclear position (the start gcode's first
            // travel): upstream's unclear-position branch emits XY
            // WITH the travel feedrate word then the separate
            // `_travel_to_z` re-statement (`GCodeWriter.cpp:768-
            // 777`: `w0.emit_xy(..); w0.emit_f(travel_speed*60)`).
            // Omitting the F word desyncs the stream feedrate —
            // the following Z's F then reads as redundant and the
            // cooling rewrite drops it.
            travel_emit::xy_with_comment(
                output,
                travel_x,
                travel_y,
                state.travel_feedrate,
                first_travel_comment,
            );
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_z(state.layer_z + state.options.z_hop),
                    format_axis(travel::lift_z_feedrate(state))
                )
                .as_bytes(),
            );
        } else {
            // Clear position: upstream `travel_to_xyz` NormalLift
            // emits the plain lift (`slop_move`) then ONE
            // combined `G1 X{} Y{} Z{}` move (`xy_z_move`,
            // `GCodeWriter.cpp:748-768`); the redundant F drops
            // in the cooling rewrite like any unchanged feedrate
            // word.
            travel_emit::xyz_with_comment(
                output,
                travel_x,
                travel_y,
                state.layer_z + state.options.z_hop,
                state.travel_feedrate,
                first_travel_comment,
            );
            state.current_feedrate = state.travel_feedrate;
        }
    } else {
        travel_emit::xy_with_comment(
            output,
            travel_x,
            travel_y,
            state.travel_feedrate,
            first_travel_comment,
        );
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z + state.options.z_hop),
                format_axis(travel::lift_z_feedrate(state))
            )
            .as_bytes(),
        );
    }
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
}
