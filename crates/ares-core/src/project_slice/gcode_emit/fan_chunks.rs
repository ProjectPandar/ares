//! FanMover chunk processing — the per-reserved-layer-tag flush loop
//! extracted from the export tail (`GCode.cpp:3749`).

use super::fan_mover::FanMover;

/// Processes the layers span through the FanMover, one flush per
/// reserved-layer-tag chunk, with the leading M106/M107 lines walked back
/// into the chunk that owns them (upstream `change_extruder_set_fan(true)`
/// prepends the fan to the rebuilt layer text).
pub(super) fn process_layer_chunks_into(
    mover: &mut FanMover,
    layers: &[u8],
    layer_change: &str,
    out: &mut Vec<u8>,
) {
    let text = std::str::from_utf8(layers).expect("generated G-code is UTF-8");
    let mut chunk_start = 0;
    for (offset, _) in text.match_indices("\n;") {
        let line_end = offset + 1;
        if text[line_end..].starts_with(layer_change) {
            let boundary = walk_back_fan_lines(text, chunk_start, line_end);
            mover.process_gcode_into(&text[chunk_start..boundary], true, out);
            chunk_start = boundary;
        }
    }
    mover.process_gcode_into(&text[chunk_start..], true, out);
}

fn walk_back_fan_lines(text: &str, chunk_start: usize, line_end: usize) -> usize {
    let mut boundary = line_end;
    let bytes = text.as_bytes();
    while boundary > chunk_start {
        let mut prev = boundary - 1;
        while prev > chunk_start && bytes[prev - 1] != b'\n' {
            prev -= 1;
        }
        if !text[prev..boundary].starts_with("M106") && !text[prev..boundary].starts_with("M107") {
            break;
        }
        boundary = prev;
    }
    boundary
}
