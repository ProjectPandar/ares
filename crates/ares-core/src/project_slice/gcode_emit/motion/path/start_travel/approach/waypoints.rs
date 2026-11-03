//! The detour-waypoint tail of the first travel (`GCode.cpp:7488-7501`):
//! upstream emits every route waypoint through `travel_to_xy` WITH the F
//! word (the cooling buffer strips unchanged feedrates or zero-length
//! duplicates), and the FINAL waypoint goes through `travel_to_xyz`
//! carrying the scarf destination z when a z ratio applies.

use crate::project_slice::gcode_emit::motion::{
    EmitState, arc,
    format::{axis as format_axis, z as format_z},
};

/// Emit `route[1..]` — the detour waypoints after the first approach
/// move — and return whether the final leg already consumed the layer z.
pub(super) fn emit_detour_waypoints(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    route: &[arc::Point],
    slope_start_z: Option<f64>,
    route_comment: &str,
) -> bool {
    let mut travel_set_layer_z = false;
    for (index, point) in route[1..].iter().enumerate() {
        let last = index + 1 == route.len() - 1;
        if last && let Some(z) = slope_start_z {
            // The final leg is an ordinary (non-forced) `travel_to_xyz`
            // (`GCode.cpp:7493-7497`): with `abs(delta(2)) < EPSILON`
            // it collapses to the commentless XY branch
            // (`GCodeWriter.cpp:927`, `libslic3r.h:52`) and keeps the
            // writer z + lift state untouched.
            // Upstream `will_move_z` (`GCodeWriter.cpp:916-928`): a
            // lifted nozzle treats a z inside [pos-lifted, pos] as
            // no-move; only WITHOUT positive lift does the bare
            // `abs(delta) < EPSILON` shortcut apply.
            let live_z = state.writer_z.unwrap_or(state.layer_z);
            let nominal_z = live_z - state.lifted_amount;
            let no_z_move = if state.lifted_amount > 0.0 {
                z >= nominal_z && z <= live_z
            } else {
                (z - live_z).abs() < 1e-4
            };
            if no_z_move {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} F{}\n",
                        format_axis(point.x - state.xy_offset.0),
                        format_axis(point.y - state.xy_offset.1),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{} F{}{route_comment}\n",
                        format_axis(point.x - state.xy_offset.0),
                        format_axis(point.y - state.xy_offset.1),
                        format_z(z),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                state.record_z(z);
                // The final leg's `travel_to_xyz` cancels the lift
                // state (`GCodeWriter.cpp:782-785`) — the nozzle
                // already reached its travel destination; without
                // this, `restate` emits a second descent to the same z.
                state.lifted = false;
                state.lifted_amount = 0.0;
                state.template_lifted = false;
                travel_set_layer_z = true;
            }
        } else {
            output.extend_from_slice(
                format!(
                    "G1 X{} Y{} F{}{route_comment}\n",
                    format_axis(point.x - state.xy_offset.0),
                    format_axis(point.y - state.xy_offset.1),
                    format_axis(state.travel_feedrate)
                )
                .as_bytes(),
            );
        }
        state.x = point.x;
        state.y = point.y;
    }
    travel_set_layer_z
}
