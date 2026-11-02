//! Shared literal centripetal limit: non-Klipper G1 and VG1
//! (`GCodeProcessor.cpp:5453-5477,5859-5883`).

pub(super) fn limit(
    delta: [f64; 4],
    previous: [f32; 4],
    direction: [f32; 4],
    acceleration: f64,
    mut feedrate: f32,
) -> f32 {
    let previous_xy = xy_unit(previous);
    let current_xy = xy_unit(direction);
    if let (Some(previous_xy), Some(current_xy)) = (previous_xy, current_xy) {
        let difference_x = current_xy[0] - previous_xy[0];
        let difference_y = current_xy[1] - previous_xy[1];
        let difference = (difference_x * difference_x + difference_y * difference_y).sqrt();
        if difference < 0.5 && difference > 0.000_01 {
            let dot = previous_xy[0] * current_xy[0] + previous_xy[1] * current_xy[1];
            let cross = previous_xy[0] * current_xy[1] - previous_xy[1] * current_xy[0];
            let angle = f64::from(cross).atan2(f64::from(dot)) as f32;
            let sin_half = ((1.0 - angle.cos()) * 0.5).sqrt();
            let radius = ((delta[0] * delta[0] + delta[1] * delta[1]).sqrt() * 0.5
                / f64::from(sin_half)) as f32;
            feedrate = feedrate.min(((acceleration as f32) * radius).sqrt());
        }
    }
    feedrate
}

fn xy_unit(direction: [f32; 4]) -> Option<[f32; 2]> {
    let norm = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
    (norm > 0.0).then(|| [direction[0] / norm, direction[1] / norm])
}
