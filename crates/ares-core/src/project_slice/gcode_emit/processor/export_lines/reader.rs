//! Command/axis boundaries from `GCodeReader.hpp:180–195` and `.cpp:292–322`.

mod number;

pub(in crate::project_slice::gcode_emit::processor) fn command(line: &str) -> &str {
    line.trim_start_matches([' ', '\t'])
        .split([' ', '\t', ';', '\r', '\n', '\0'])
        .next()
        .unwrap_or_default()
}

pub(super) fn axis(line: &str, axis: char) -> Option<f32> {
    let code = line.split([';', '\r', '\n', '\0']).next()?;
    let mut rest = code.trim_start_matches([' ', '\t']);
    rest = rest.split_once([' ', '\t']).map_or("", |(_, tail)| tail);
    while !rest.is_empty() {
        rest = rest.trim_start_matches([' ', '\t']);
        if let Some(value) = rest.strip_prefix(axis) {
            let word = value
                .trim_start_matches(|character: char| character.is_ascii_whitespace())
                .split([' ', '\t'])
                .next()
                .unwrap_or_default();
            if let Some(value) = number::parse(word) {
                return Some(value as f32);
            }
            // strtod leaves its end pointer at the original input on failure.
            if !number::consumes_prefix(word)
                && (value.is_empty() || value.starts_with([' ', '\t']))
            {
                return Some(0.0);
            }
        }
        rest = rest.split_once([' ', '\t']).map_or("", |(_, tail)| tail);
    }
    None
}

pub(super) fn is_start(command: &str) -> bool {
    ["G28", "G29", "PRINT_START", "START_PRINT"]
        .iter()
        .any(|start| command.eq_ignore_ascii_case(start))
}
