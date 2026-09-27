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
    // The lift target is `m_pos.z() + m_to_lift` (`GCodeWriter.cpp:
    // 701-709`) — the LIVE writer z (template `position[2]` write-back
    // can move it off the nominal layer z).
    let raised_z = state.writer_z.unwrap_or(state.layer_z) + state.options.z_hop;
    let mode = first_travel_lift.unwrap_or_else(|| travel::lift_mode_for(state, true));
    // `travel_to_xyz` NormalLift emits the lift via
    // `_travel_to_z(.., "normal lift Z")` (`GCodeWriter.cpp:678`),
    // trailing the ` ; normal lift Z` comment under `gcode_comments`.
    let lift_comment = state
        .options
        .gcode_comments
        .then_some(" ; normal lift Z")
        .unwrap_or("");
    if mode == LiftMode::Normal {
        let feedrate = travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}{lift_comment}\n",
                format_z(raised_z),
                format_axis(feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = feedrate;
        state.record_z(raised_z);
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
            // `_travel_to_z(.., comment)` repeats the travel comment on the
            // re-statement (`GCodeWriter.cpp:777`).
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{first_travel_comment}\n",
                    format_z(raised_z),
                    format_axis(travel::lift_z_feedrate(state))
                )
                .as_bytes(),
            );
            state.record_z(raised_z);
        } else {
            // Clear position with a pending lift: upstream emits the
            // lift, then `travel_to_xy`-style XY WITHOUT a Z word (the
            // covered Z does not move, `GCodeWriter.cpp:769-780`), then
            // the `_travel_to_z` re-statement of the raised z. Both
            // legs carry their feedrate + comment like the unclear
            // branch — CoolingBuffer drops redundant F words and
            // deletes zero-length moves (`CoolingBuffer.cpp:901-920`).
            travel_emit::xy_with_comment(
                output,
                travel_x,
                travel_y,
                state.travel_feedrate,
                first_travel_comment,
            );
            state.current_feedrate = state.travel_feedrate;
            state.record_z(raised_z);
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{first_travel_comment}\n",
                    format_z(raised_z),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
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
                "G1 Z{} F{}{first_travel_comment}\n",
                format_z(raised_z),
                format_axis(travel::lift_z_feedrate(state))
            )
            .as_bytes(),
        );
        state.record_z(raised_z);
    }
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
}
