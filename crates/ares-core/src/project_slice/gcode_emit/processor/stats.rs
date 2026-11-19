//! Deferred statistics registered by `GCode.cpp:4111–4126`, replaced in
//! `GCodeProcessor.cpp:1370–1422`.

use std::borrow::Cow;

use super::{material_statistics::MaterialStatistics, time};

pub(in crate::project_slice::gcode_emit) const DEFERRED_PLACEHOLDERS: [(&[&str], &str); 12] = [
    (&["print_time_total_sec"], "@PRINT_TIME_TOTAL_SEC@"),
    (&["print_time_day"], "@PRINT_TIME_DAY@"),
    (&["print_time_hour"], "@PRINT_TIME_HOUR@"),
    (&["print_time_minute"], "@PRINT_TIME_MINUTE@"),
    (&["print_time_sec"], "@PRINT_TIME_SEC@"),
    (
        &["used_filament_length", "used_filament"],
        "@USED_FILAMENT_LENGTH@",
    ),
    (&["print_time", "normal_print_time"], "@NORMAL_PRINT_TIME@"),
    (&["silent_print_time"], "@SILENT_PRINT_TIME@"),
    (&["total_weight"], "@TOTAL_WEIGHT@"),
    (&["total_cost"], "@TOTAL_COST@"),
    (&["total_wipe_tower_cost"], "@TOTAL_WIPE_TOWER_COST@"),
    (
        &["total_wipe_tower_filament"],
        "@TOTAL_WIPE_TOWER_FILAMENT@",
    ),
];

pub(super) fn values(
    print_time: f64,
    material: &MaterialStatistics,
    time_cost: f64,
    (tower_mm, tower_cost): (f64, f64),
) -> [String; DEFERRED_PLACEHOLDERS.len()] {
    let total = print_time.max(0.0);
    let filament_m = material.length_mm / 1_000.0;
    let weight = material.weight_g;
    let cost = material.cost + time_cost * total / 3_600.0;
    let seconds = total as i32;
    let day_remainder = seconds % 86_400;
    [
        format!("{total:.2}"),
        (seconds / 86_400).to_string(),
        (day_remainder / 3_600).to_string(),
        (day_remainder % 3_600 / 60).to_string(),
        (day_remainder % 60).to_string(),
        format!("{filament_m:.2}"),
        time::short_duration(print_time),
        // The pinned exporter only enables Normal; short_time("N/A") is "0s".
        "0s".to_owned(),
        format!("{weight:.2}"),
        format!("{cost:.2}"),
        format!("{tower_cost:.2}"),
        format!("{tower_mm:.2}"),
    ]
}

pub(super) fn expand<'a>(
    line: &'a str,
    values: &[String; DEFERRED_PLACEHOLDERS.len()],
) -> Cow<'a, str> {
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
