//! GCode.cpp:677-756 Wipe::wipe movement, independent of retraction bookkeeping.

use super::super::format::{axis as format_axis, extrusion as format_extrusion};
use super::super::{extrusion, state::EmitState};
use super::wipe_path::{WipePath, wipe_moves};
use super::wipe_sequence::gcode_comment;

pub(in crate::project_slice::gcode_emit::motion) fn without_retraction(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    let path = wipe_moves(state);
    let speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    emit(output, state, path, (0.0, speed, true));
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    WipePath {
        segments,
        distribution_distance,
        ..
    }: WipePath,
    (during, wipe_speed, cooling_marker): (f64, f64, bool),
) {
    let wipe_start = if state.tags.is_bbl() {
        "; WIPE_START\n"
    } else {
        ";WIPE_START\n"
    };
    output.extend_from_slice(wipe_start.as_bytes());
    let speed_line = if cooling_marker {
        format!("G1 F{};_WIPE\n", format_axis(wipe_speed * 60.0))
    } else {
        format!("G1 F{}\n", format_axis(wipe_speed * 60.0))
    };
    output.extend_from_slice(speed_line.as_bytes());
    state.current_feedrate = wipe_speed * 60.0;
    for (point, segment_length) in segments {
        let retraction = during * (segment_length / distribution_distance);
        // Orca's `extrude_to_xy` omits the E word when the wipe carries
        // no retraction (`retract_before_wipe` = 100% leaves nothing to
        // distribute during the wipe).
        let line = if retraction > f64::EPSILON {
            let retract = extrusion::coordinate(state, -retraction);
            let comment = gcode_comment(state, " ; wipe and retract");
            format!(
                "G1 X{} Y{} E{}{comment}\n",
                format_axis(point.x - state.xy_offset.0),
                format_axis(point.y - state.xy_offset.1),
                format_extrusion(retract)
            )
        } else {
            let comment = gcode_comment(state, " ; wipe and retract");
            format!(
                "G1 X{} Y{}{comment}\n",
                format_axis(point.x - state.xy_offset.0),
                format_axis(point.y - state.xy_offset.1)
            )
        };
        output.extend_from_slice(line.as_bytes());
        state.x = point.x;
        state.y = point.y;
        // Recover the scaled-int position the mm value came from
        // (upstream wipe moves hold ints; mm→int round-trips are
        // exact under round-to-nearest).
        // `(point - m_origin) + extruder_offset` order
        // (`GCode.cpp:8138-8144`).
        state.last_scaled_position = (
            ((point.x - state.origin.0 + state.extruder_offset.0) / state.scale_factor).round()
                as i64,
            ((point.y - state.origin.1 + state.extruder_offset.1) / state.scale_factor).round()
                as i64,
        );
        state.last_position_defined = true;
        state.wipe_start = Some(point);
    }
    let wipe_end = if state.tags.is_bbl() {
        "; WIPE_END\n"
    } else {
        ";WIPE_END\n"
    };
    output.extend_from_slice(wipe_end.as_bytes());
    // `Wipe::wipe` resets the stored path so a subsequent retract
    // does not re-wipe the same path (`GCode.cpp:497`).
    state.wipe_path.clear();
}
