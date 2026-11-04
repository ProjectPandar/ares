//! `custom_gcode_motion_state_changes` (`GCode.cpp:842-876`) and the
//! writer invalidation `placeholder_parser_process` applies after it
//! (`GCode.cpp:5362-5366`, `GCodeWriter.hpp:165-166`).

use super::EmitState;

/// `GCodeReader::is_end_of_line` (`GCodeReader.hpp:190`).
fn is_end_of_line(c: u8) -> bool {
    matches!(c, b'\r' | b'\n' | 0)
}

/// `GCodeLine::cmd` (`GCodeReader.hpp:27-30`): the first word after the
/// leading blanks, ending at a blank, `;` or the end of the line
/// (`GCodeReader.hpp:189-200`).
fn cmd(raw: &[u8]) -> &[u8] {
    let start = raw
        .iter()
        .position(|&c| !matches!(c, b' ' | b'\t'))
        .unwrap_or(raw.len());
    let rest = &raw[start..];
    let len = rest
        .iter()
        .position(|&c| matches!(c, b' ' | b'\t' | b';'))
        .unwrap_or(rest.len());
    &rest[..len]
}

/// `custom_gcode_line_has_xy_parameter` (`GCode.cpp:848-853`).
fn has_xy_parameter(raw: &[u8]) -> bool {
    let code = raw.split(|&c| c == b';').next().unwrap_or_default();
    code.iter().any(|c| matches!(c, b'X' | b'x' | b'Y' | b'y'))
}

/// `boost::icontains` on the raw line.
fn icontains(raw: &[u8], needle: &[u8]) -> bool {
    raw.windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

/// Drops the writer's cached acceleration and jerk when the expanded
/// custom G-code sets them itself, so the next `M204`/`M205` is written.
pub(in crate::project_slice::gcode_emit) fn invalidate_after_custom_gcode(
    gcode: &str,
    state: &mut EmitState,
) {
    let (mut acceleration, mut jerk) = (false, false);
    let bytes = gcode.as_bytes();
    let mut ptr = 0;
    // `GCodeReader::parse_buffer` (`GCodeReader.hpp:119-129`) with the line
    // split of `parse_line_internal` (`GCodeReader.cpp:113-127`).
    while ptr < bytes.len() && bytes[ptr] != 0 && !(acceleration && jerk) {
        let end = bytes[ptr..]
            .iter()
            .position(|&c| is_end_of_line(c))
            .map_or(bytes.len(), |len| ptr + len);
        let raw = &bytes[ptr..end];
        let cmd = cmd(raw);
        let is = |name: &str| cmd.eq_ignore_ascii_case(name.as_bytes());
        if is("M204") || is("M201") || is("M202") {
            acceleration = true;
        } else if (is("M205") || is("M207") || is("M566")) && has_xy_parameter(raw) {
            jerk = true;
        } else if is("SET_VELOCITY_LIMIT") {
            acceleration |= icontains(raw, b"ACCEL=");
            jerk |= icontains(raw, b"SQUARE_CORNER_VELOCITY=");
        }
        ptr = end;
        if bytes.get(ptr) == Some(&b'\r') {
            ptr += 1;
        }
        if bytes.get(ptr) == Some(&b'\n') {
            ptr += 1;
        }
    }
    if acceleration {
        state.last_acceleration = None;
        state.last_travel_acceleration = None;
    }
    if jerk {
        state.last_jerk = None;
    }
}
