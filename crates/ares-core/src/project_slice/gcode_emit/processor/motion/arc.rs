use super::word;

pub(super) struct ArcMotion {
    pub(super) start: [f64; 3],
    pub(super) end: [f64; 3],
    pub(super) e_delta: f64,
    pub(super) feedrate: f64,
}

pub(super) fn deltas(command: &str, code: &str, motion: ArcMotion) -> Option<Vec<[f64; 4]>> {
    let ArcMotion {
        start,
        end,
        e_delta,
        feedrate,
    } = motion;
    let i = word(code, 'I').unwrap_or(0.0) as f32;
    let j = word(code, 'J').unwrap_or(0.0) as f32;
    let radius = i.hypot(j);
    if radius <= f32::EPSILON {
        return None;
    }
    let start = start.map(|value| value as f32);
    let end = end.map(|value| value as f32);
    let center = [start[0] + i, start[1] + j];
    let start_radius = [start[0] - center[0], start[1] - center[1]];
    let end_radius = [end[0] - center[0], end[1] - center[1]];
    let same_xy =
        (end[0] - start[0]).abs() <= f32::EPSILON && (end[1] - start[1]).abs() <= f32::EPSILON;
    let mut sweep = if same_xy {
        0.0_f64
    } else {
        let cross = f64::from(start_radius[0]) * f64::from(end_radius[1])
            - f64::from(start_radius[1]) * f64::from(end_radius[0]);
        let dot = f64::from(start_radius[0]) * f64::from(end_radius[0])
            + f64::from(start_radius[1]) * f64::from(end_radius[1]);
        let mut angle = cross.atan2(dot);
        if angle < 0.0 {
            angle += 2.0 * std::f64::consts::PI;
        }
        if command == "G2" {
            angle -= 2.0 * std::f64::consts::PI;
        }
        angle
    };
    let turns = word(code, 'P').unwrap_or(0.0) as f32;
    let turns = if same_xy && turns == 0.0 { 1.0 } else { turns } * 2.0 * std::f32::consts::PI;
    sweep += f64::from(if command == "G2" { -turns } else { turns });
    let segment_mm = (8.0_f32 * radius * 0.02)
        .sqrt()
        .min(feedrate as f32 / 50.0)
        .clamp(0.1, 2.0);
    let segments = (f64::from(radius) * sweep.abs() / f64::from(segment_mm) + 0.8) as usize;
    let segments = segments.max(1);
    let theta = sweep as f32 / segments as f32;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let z_step = (end[2] - start[2]) / segments as f32;
    let e_step = e_delta as f32 / segments as f32;
    let mut rvec = [-i, -j];
    let mut correction = 25;
    let mut previous = [start[0], start[1], start[2], 0.0_f32];
    let mut deltas = Vec::with_capacity(segments);
    for segment in 1..segments {
        correction -= 1;
        if correction == 0 {
            correction = 25;
            let angle = theta * segment as f32;
            let (sin, cos) = angle.sin_cos();
            rvec = [-i * cos + j * sin, -i * sin - j * cos];
        } else {
            let new_y = rvec[0] * sin_theta + rvec[1] * cos_theta;
            rvec[0] = rvec[0] * cos_theta - rvec[1] * sin_theta;
            rvec[1] = new_y;
        }
        let current = [
            center[0] + rvec[0],
            center[1] + rvec[1],
            previous[2] + z_step,
            previous[3] + e_step,
        ];
        deltas.push([
            f64::from(current[0] - previous[0]),
            f64::from(current[1] - previous[1]),
            f64::from(current[2] - previous[2]),
            f64::from(current[3] - previous[3]),
        ]);
        previous = current;
    }
    let current = [end[0], end[1], end[2], e_delta as f32];
    deltas.push([
        f64::from(current[0] - previous[0]),
        f64::from(current[1] - previous[1]),
        f64::from(current[2] - previous[2]),
        f64::from(current[3] - previous[3]),
    ]);
    Some(deltas)
}
