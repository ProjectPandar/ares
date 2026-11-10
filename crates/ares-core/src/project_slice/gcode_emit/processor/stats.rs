//! Deferred print time placeholders: `GCode.cpp:4086-4091` sets the reserved tags
//! (`GCodeProcessor.cpp:120-125`), replaced on every exported line by `:1385-1407`.

use std::borrow::Cow;

pub(in crate::project_slice::gcode_emit) const DEFERRED_PLACEHOLDERS: [(&str, &str); 6] = [
    ("print_time_total_sec", "@PRINT_TIME_TOTAL_SEC@"),
    ("print_time_day", "@PRINT_TIME_DAY@"),
    ("print_time_hour", "@PRINT_TIME_HOUR@"),
    ("print_time_minute", "@PRINT_TIME_MINUTE@"),
    ("print_time_sec", "@PRINT_TIME_SEC@"),
    ("used_filament_length", "@USED_FILAMENT_LENGTH@"),
];

/// Replacement values in `DEFERRED_PLACEHOLDERS` order, from the normal mode time in seconds
/// and the total filament length in millimetres.
pub(super) fn values(print_time: f64, filament_mm: f64) -> [String; 6] {
    let total = print_time.max(0.0);
    let seconds = total as i32;
    let day_remainder = seconds % 86_400;
    [
        format!("{total:.2}"),
        (seconds / 86_400).to_string(),
        (day_remainder / 3_600).to_string(),
        (day_remainder % 3_600 / 60).to_string(),
        (day_remainder % 60).to_string(),
        format!("{:.2}", filament_mm / 1_000.0),
    ]
}

pub(super) fn expand<'a>(line: &'a str, values: &[String; 6]) -> Cow<'a, str> {
    let mut expanded = Cow::Borrowed(line);
    if !line.contains('@') {
        return expanded;
    }
    for ((_, tag), value) in DEFERRED_PLACEHOLDERS.iter().zip(values) {
        if expanded.contains(tag) {
            expanded = Cow::Owned(expanded.replace(tag, value));
        }
    }
    expanded
}
