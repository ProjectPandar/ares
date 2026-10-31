//! `GCode.cpp::WipeTowerIntegration::post_process_wipe_tower_moves`.
use super::result::ToolChangeResult;

pub(super) fn transform(point: [f32; 2], translation: [f32; 2], angle: f32) -> [f32; 2] {
    let (sin, cos) = angle.sin_cos();
    [
        cos * point[0] - sin * point[1] + translation[0],
        sin * point[0] + cos * point[1] + translation[1],
    ]
}

pub(super) fn moves(
    result: &ToolChangeResult,
    translation: [f32; 2],
    angle: f32,
    offsets: &[[f32; 2]],
    semm: bool,
) -> String {
    let mut extruder_offset = offsets[if semm { 0 } else { result.initial_tool }];
    let mut position = result.start;
    let mut transformed = transform(position, translation, angle);
    let mut old = [-1000.1; 2];
    let mut output = String::with_capacity(result.gcode.len());
    for original in result.gcode.split('\n') {
        let mut line = original.to_owned();
        if line.starts_with("G1 ") || line.starts_with("G2 ") || line.starts_with("G3 ") {
            let command = line[..3].to_owned();
            let tag = "_GCODE_WIPE_TOWER_NEVER_SKIP_TAG";
            let never_skip = if let Some(index) = line.find(tag) {
                // C++ erase's second argument is a count, not an end iterator.
                line.truncate(index);
                true
            } else {
                false
            };
            line = strip_coordinates(&line, &mut position);
            transformed = transform(position, translation, angle);
            if transformed != old || never_skip {
                let replacement =
                    motion_prefix(command, transformed, old, extruder_offset, never_skip);
                line.replace_range(..3, &replacement);
                old = transformed;
            }
        }
        output.push_str(&line);
        output.push('\n');
        if line != "[change_filament_gcode]" {
            continue;
        }
        if !semm {
            extruder_offset = offsets[result.new_tool];
            if extruder_offset != offsets[result.initial_tool] {
                use std::fmt::Write as _;
                writeln!(
                    output,
                    "G1 X{:.3} Y{:.3}",
                    transformed[0] - extruder_offset[0],
                    transformed[1] - extruder_offset[1]
                )
                .unwrap();
            }
        }
        old = [-1000.1; 2];
        position = result
            .type1
            .as_ref()
            .map_or(result.start, |metadata| metadata.change_start);
        transformed = transform(position, translation, angle);
    }
    output
}

/// `GCode.cpp:364–412`: nozzle-change stream transformation (G1 only).
pub(super) fn nozzle_moves(
    gcode: &str,
    mut position: [f32; 2],
    translation: [f32; 2],
    angle: f32,
) -> String {
    let mut old = [-1000.1; 2];
    let mut output = String::with_capacity(gcode.len());
    for original in gcode.split('\n') {
        let mut line = original.to_owned();
        if line.starts_with("G1 ") {
            let never_skip = if let Some(index) = line.find("_GCODE_WIPE_TOWER_NEVER_SKIP_TAG") {
                line.truncate(index);
                true
            } else {
                false
            };
            let stripped = strip_coordinates(&line, &mut position);
            let transformed = transform(position, translation, angle);
            if transformed != old || never_skip {
                line = stripped;
                line.replace_range(
                    ..3,
                    &motion_prefix("G1 ".into(), transformed, old, [0.0; 2], never_skip),
                );
                old = transformed;
            }
        }
        output.push_str(&line);
        output.push('\n');
    }
    output
}

fn strip_coordinates(line: &str, position: &mut [f32; 2]) -> String {
    let bytes = line.as_bytes();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let ch = bytes[index];
        index += 1;
        if matches!(ch, b'X' | b'Y') {
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_digit()
                    || matches!(bytes[index], b'+' | b'-' | b'.' | b'e' | b'E'))
            {
                index += 1;
            }
            position[usize::from(ch == b'Y')] =
                line[start..index].parse().expect("tower writer coordinate");
        } else {
            stripped.push(ch);
        }
    }
    String::from_utf8(stripped).expect("removing ASCII coordinates preserves UTF-8")
}

fn motion_prefix(
    command: String,
    transformed: [f32; 2],
    old: [f32; 2],
    extruder_offset: [f32; 2],
    never_skip: bool,
) -> String {
    use std::fmt::Write as _;
    let mut replacement = command;
    if transformed[0] != old[0] || never_skip {
        write!(replacement, " X{:.3}", transformed[0] - extruder_offset[0]).unwrap();
    }
    if transformed[1] != old[1] || never_skip {
        write!(replacement, " Y{:.3}", transformed[1] - extruder_offset[1]).unwrap();
    }
    replacement.push(' ');
    replacement
}
