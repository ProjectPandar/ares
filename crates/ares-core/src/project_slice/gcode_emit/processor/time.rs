fn parts(seconds: f64) -> (u64, u64, u64, f32) {
    let mut remaining = seconds as f32;
    let days = (remaining / 86400.0) as u64;
    remaining -= days as f32 * 86400.0;
    let hours = (remaining / 3600.0) as u64;
    remaining -= hours as f32 * 3600.0;
    let minutes = (remaining / 60.0) as u64;
    remaining -= minutes as f32 * 60.0;
    (days, hours, minutes, remaining)
}

/// `Utils.hpp get_time_dhms`: days/hours/minutes decompose the f32 seconds;
/// values at or below one second print with the `%f` fractional format.
pub(super) fn duration(seconds: f64) -> String {
    let (days, hours, minutes, remaining) = parts(seconds);
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

/// `Utils.hpp:573–642`: shorten `get_time_dhms`; round to minutes only
/// when the duration already contains days or hours.
pub(super) fn short_duration(seconds: f64) -> String {
    let (mut days, mut hours, mut minutes, remaining) = parts(seconds);
    let whole = remaining as u64;
    if (days > 0 || hours > 0) && whole >= 30 {
        minutes += 1;
        if minutes == 60 {
            minutes = 0;
            hours += 1;
            if hours == 24 {
                hours = 0;
                days += 1;
            }
        }
    }
    if days > 0 {
        format!("{days}d{hours}h{minutes}m")
    } else if hours > 0 {
        format!("{hours}h{minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m{whole}s")
    } else if whole > 0 {
        format!("{whole}s")
    } else {
        // short_time reads back the six-place %f for subsecond values.
        match format!("{remaining:.6}").as_str() {
            "0.000000" => "0s".to_owned(),
            "1.000000" => "1s".to_owned(),
            _ => "<1s".to_owned(),
        }
    }
}

pub(super) fn minutes(seconds: f64) -> u64 {
    // Upstream `time_in_minutes` (`GCodeProcessor.cpp:1013-1016`) performs the
    // +0.5 offset and the /60 division in f32: int((t + 0.5f) / 60.0f).
    ((seconds as f32 + 0.5_f32) / 60.0_f32) as u64
}
