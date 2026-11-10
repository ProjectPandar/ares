//! `process_custom_gcode_time`, `finalize`, `get_custom_gcode_times`
//! (GCodeProcessor.cpp:5917-5934, 2702-2707, 2737-2750).

use crate::gcode_processor::{CustomGCodeTime, CustomGCodeType};

pub(super) fn custom_times(
    events: &[(CustomGCodeType, usize)],
    flush_ends: &[usize],
    times: &[f64],
    trailing_delay: f64,
    total: f64,
) -> Vec<CustomGCodeTime> {
    if events.is_empty() {
        return Vec::new();
    }
    let mut records = Vec::with_capacity(events.len() + 1);
    let mut cursor = 0;
    for &(kind, flush) in events {
        // A synchronize may retain a single pending block. Count only blocks
        // actually processed by that flush, not every block parsed so far.
        let end = flush_ends[flush];
        let duration = times[cursor..end]
            .iter()
            .fold(0.0_f32, |cache, time| (f64::from(cache) + time) as f32);
        cursor = end;
        if duration != 0.0 {
            records.push((kind, duration));
        }
    }
    let trailing = times[cursor..]
        .iter()
        .fold(0.0_f32, |cache, time| (f64::from(cache) + time) as f32)
        + trailing_delay as f32;
    if trailing != 0.0 {
        records.push((CustomGCodeType::ColorChange, trailing));
    }
    let mut elapsed = 0.0_f32;
    records
        .into_iter()
        .map(|(kind, duration)| {
            let remaining = (total - f64::from(elapsed)) as f32;
            elapsed += duration;
            CustomGCodeTime {
                kind,
                duration,
                remaining,
            }
        })
        .collect()
}
