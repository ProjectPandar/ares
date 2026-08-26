pub(super) fn minutes(seconds: f64) -> u64 {
    (((seconds as f32) + 0.5) / 60.0) as u64
}

pub(super) fn duration(seconds: f64) -> String {
    let mut remaining = seconds as f32 as u64;
    let hours = remaining / 3600;
    remaining %= 3600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
