//! First-travel emit dispatch branches — the `travel_to_xyz` lift
//! variants (`GCodeWriter.cpp:685-806`).

use super::super::travel_emit;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LiftMode,
    format::{axis as format_axis, z as format_z},
    travel,
};

/// The lifted first-travel description shared by the dispatch branch
/// (`GCodeWriter.cpp:748-777`): XY destination and comment, the
/// scheduled lift mode, whether the source position is genuinely
/// unknown, and the travel's destination z.
pub(super) struct FirstTravel<'a> {
    pub(super) travel_x: f64,
    pub(super) travel_y: f64,
    pub(super) comment: &'a str,
    pub(super) lift: Option<LiftMode>,
    pub(super) never_positioned: bool,
    pub(super) dest_z: f64,
}

/// `state.extruder.retracted && first_position && z_hop > 0` branch of
/// `emit_needs_travel`: emit the Z lift, then the approach move whose
/// shape depends on whether the source position is genuinely unknown
/// (`GCodeWriter.cpp:748-777`).
pub(super) fn emit_retracted_first_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    travel: FirstTravel<'_>,
) {
    let FirstTravel {
        travel_x,
        travel_y,
        comment: first_travel_comment,
        lift: first_travel_lift,
        never_positioned,
        dest_z,
    } = travel;
    // The lift target is `m_pos.z() + m_to_lift` (`GCodeWriter.cpp:
    // 701-709`) — the LIVE writer z (template `position[2]` write-back
    // can move it off the nominal layer z), but only while it exceeds
    // the ACTUAL destination z (`point(2)` — a scarf seam's slope start,
    // not necessarily the nominal layer z, `GCode.cpp:7481-7482`).
    let live_z = state.writer_z.unwrap_or(state.layer_z);
    let raised_z = if live_z + state.options.z_hop > dest_z {
        live_z + state.options.z_hop
    } else {
        dest_z
    };
    let mode = first_travel_lift.unwrap_or_else(|| travel::lift_mode_for(state, true));
    // `travel_to_xyz` NormalLift emits the lift via
    // `_travel_to_z(.., "normal lift Z")` (`GCodeWriter.cpp:678`),
    // trailing the ` ; normal lift Z` comment under `gcode_comments`.
    let lift_comment = if state.options.gcode_comments {
        " ; normal lift Z"
    } else {
        ""
    };
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
                state,
                (travel_x, travel_y),
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
            // Clear position: upstream's `xy_z_move` is ONE combined
            // `G1 X Y Z` move to the raised destination
            // (`GCodeWriter.cpp:748-753`).
            travel_emit::xyz_with_comment(
                output,
                state,
                crate::Point3d {
                    x: travel_x,
                    y: travel_y,
                    z: raised_z,
                },
                state.travel_feedrate,
                first_travel_comment,
            );
            state.current_feedrate = state.travel_feedrate;
            state.record_z(raised_z);
        }
    } else {
        travel_emit::xy_with_comment(
            output,
            state,
            (travel_x, travel_y),
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
    // The lift exists only when the raised destination exceeds the
    // travel's destination z (`GCodeWriter.cpp:705-710`).
    state.lifted = raised_z > dest_z;
    state.lifted_amount = (raised_z - dest_z).max(0.0);
}
