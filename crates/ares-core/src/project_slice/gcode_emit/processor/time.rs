/// `Utils.hpp get_time_dhms`: days/hours/minutes decompose the f32 seconds;
/// values at or below one second print with the `%f` fractional format.
pub(super) fn duration(seconds: f64) -> String {
    let mut remaining = seconds as f32;
    let days = (remaining / 86400.0) as u64;
    remaining -= days as f32 * 86400.0;
    let hours = (remaining / 3600.0) as u64;
    remaining -= hours as f32 * 3600.0;
    let minutes = (remaining / 60.0) as u64;
    remaining -= minutes as f32 * 60.0;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m {}s", remaining as u64)
    } else if hours > 0 {
        format!("{hours}h {minutes}m {}s", remaining as u64)
    } else if minutes > 0 {
        format!("{minutes}m {}s", remaining as u64)
    } else if remaining > 1.0 {
        format!("{}s", remaining as u64)
    } else {
        format!("{remaining:.6}s")
    }
}

pub(super) fn minutes(seconds: f64) -> u64 {
    // Upstream `time_in_minutes` (`GCodeProcessor.cpp:1013-1016`) performs the
    // +0.5 offset and the /60 division in f32: int((t + 0.5f) / 60.0f).
    ((seconds as f32 + 0.5_f32) / 60.0_f32) as u64
}
