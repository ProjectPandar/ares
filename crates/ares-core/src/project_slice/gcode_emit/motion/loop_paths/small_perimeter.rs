use super::super::MotionOptions;
use super::ExtrusionPath;

// `ExtrusionLoop::length`, `GCode.cpp:5808-5813`, `libslic3r.h:84`.
pub(super) fn speed(paths: &[ExtrusionPath], options: &MotionOptions, scale: f64) -> Option<f64> {
    let length = paths
        .iter()
        .map(|path| {
            path.polyline
                .points
                .windows(2)
                .map(|pair| {
                    let dx = (pair[1].x - pair[0].x) as f64;
                    let dy = (pair[1].y - pair[0].y) as f64;
                    let dz = (pair[1].z - pair[0].z) as f64;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                })
                .sum::<f64>()
        })
        .sum::<f64>();
    // Use the complete unclipped scaled length, not per-path mm lengths.
    (length <= (options.small_perimeter_threshold / scale) * 2.0 * std::f64::consts::PI)
        .then_some({
            if options.small_perimeter_speed == 0.0 {
                options.outer_wall_speed * 0.5
            } else {
                options.small_perimeter_speed
            }
        })
        .filter(|speed| *speed > 0.0)
}
