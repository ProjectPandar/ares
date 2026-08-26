use super::word;
use crate::options::GCodeFlavor;

pub(super) struct ArcMotion {
    pub(super) start: [f64; 3],
    pub(super) end: [f64; 3],
    pub(super) e_delta: f64,
    pub(super) feedrate: f64,
    pub(super) gcode_flavor: GCodeFlavor,
}

pub(super) fn deltas(command: &str, code: &str, motion: ArcMotion) -> Option<Vec<[f64; 4]>> {
    let ArcMotion {
        start,
        end,
        e_delta,
        feedrate,
        gcode_flavor,
    } = motion;
    let i = word(code, 'I').unwrap_or(0.0) as f32;
    let j = word(code, 'J').unwrap_or(0.0) as f32;
    let radius = (i * i + j * j).sqrt();
    if radius <= f32::EPSILON {
        return None;
    }

    let center = [start[0] + f64::from(i), start[1] + f64::from(j)];
    let start_radius = [start[0] - center[0], start[1] - center[1]];
    let end_radius = [end[0] - center[0], end[1] - center[1]];
    let full_circle = (end[0] - start[0]).abs() < 1.0e-4 && (end[1] - start[1]).abs() < 1.0e-4;
    let sweep = if full_circle {
        std::f64::consts::TAU
    } else {
        let cross = start_radius[0] * end_radius[1] - start_radius[1] * end_radius[0];
        let dot = start_radius[0] * end_radius[0] + start_radius[1] * end_radius[1];
        let mut angle = cross.atan2(dot);
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }
        if command == "G2" {
            angle -= std::f64::consts::TAU;
        }
        angle
    };
    let arc = ArcGeometry {
        start,
        end,
        center,
        start_radius,
        e_delta,
        feedrate,
        i,
        j,
        radius,
        sweep,
    };
    Some(if gcode_flavor == GCodeFlavor::MarlinFirmware {
        marlin_deltas(arc)
    } else {
        legacy_deltas(arc)
    })
}

#[derive(Clone, Copy)]
struct ArcGeometry {
    start: [f64; 3],
    end: [f64; 3],
    center: [f64; 2],
    start_radius: [f64; 2],
    e_delta: f64,
    feedrate: f64,
    i: f32,
    j: f32,
    radius: f32,
    sweep: f64,
}

fn marlin_deltas(arc: ArcGeometry) -> Vec<[f64; 4]> {
    let segment_mm = (8.0_f32 * arc.radius * 0.02)
        .sqrt()
        .min(arc.feedrate as f32 * (1.0 / 50.0))
        .clamp(0.1, 2.0);
    let flat_mm = (f64::from(arc.radius) * arc.sweep.abs()) as f32;
    let segments = ((flat_mm / segment_mm + 0.8) as usize).max(1);
    let inv_segments = 1.0_f32 / segments as f32;
    let theta = arc.sweep as f32 * inv_segments;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let z_step = ((arc.end[2] - arc.start[2]) * f64::from(inv_segments)) as f32;
    let e_step = arc.e_delta as f32 * inv_segments;
    let mut rvec = [-arc.i, -arc.j];
    let mut correction = 25;
    let mut previous = [arc.start[0], arc.start[1], arc.start[2], 0.0];
    let mut z = arc.start[2];
    let mut e = 0.0;
    let mut deltas = Vec::with_capacity(segments);

    for segment in 1..segments {
        correction -= 1;
        if correction == 0 {
            correction = 25;
            let angle = theta * segment as f32;
            let (sin, cos) = angle.sin_cos();
            rvec = [-arc.i * cos + arc.j * sin, -arc.i * sin - arc.j * cos];
        } else {
            let new_y = rvec[0] * sin_theta + rvec[1] * cos_theta;
            rvec[0] = rvec[0] * cos_theta - rvec[1] * sin_theta;
            rvec[1] = new_y;
        }
        z += f64::from(z_step);
        e += f64::from(e_step);
        let current = [
            arc.center[0] + f64::from(rvec[0]),
            arc.center[1] + f64::from(rvec[1]),
            z,
            e,
        ];
        deltas.push([
            current[0] - previous[0],
            current[1] - previous[1],
            current[2] - previous[2],
            current[3] - previous[3],
        ]);
        previous = current;
    }
    let current = [arc.end[0], arc.end[1], arc.end[2], arc.e_delta];
    deltas.push([
        current[0] - previous[0],
        current[1] - previous[1],
        current[2] - previous[2],
        current[3] - previous[3],
    ]);
    deltas
}

fn legacy_deltas(arc: ArcGeometry) -> Vec<[f64; 4]> {
    let radius = arc.start_radius[0].hypot(arc.start_radius[1]);
    let segments = arc_discretization_steps(radius, arc.sweep.abs(), 0.0125);
    let inv_segments = 1.0 / segments as f64;
    let theta = arc.sweep * inv_segments;
    let z_step = (arc.end[2] - arc.start[2]) * inv_segments;
    let e_step = f64::from(arc.e_delta as f32) * inv_segments;
    let squared_theta = theta * theta;
    let cos_theta = 1.0 - 0.5 * squared_theta;
    let sin_theta = theta - squared_theta * theta / 6.0;
    let mut radius_vector = arc.start_radius;
    let mut correction = 25;
    let mut previous = [arc.start[0], arc.start[1], arc.start[2], 0.0];
    let mut z = arc.start[2];
    let mut e = 0.0;
    let mut deltas = Vec::with_capacity(segments);

    for segment in 1..segments {
        if correction == 0 {
            let angle = segment as f64 * theta;
            let (sin, cos) = angle.sin_cos();
            radius_vector = [
                -f64::from(arc.i) * cos + f64::from(arc.j) * sin,
                -f64::from(arc.i) * sin - f64::from(arc.j) * cos,
            ];
            correction = 25;
        } else {
            correction -= 1;
            let new_y = (radius_vector[0] * sin_theta + radius_vector[1] * cos_theta) as f32;
            radius_vector[0] = radius_vector[0] * cos_theta - radius_vector[1] * sin_theta;
            radius_vector[1] = f64::from(new_y);
        }
        z += z_step;
        e += e_step;
        let current = [
            arc.center[0] + radius_vector[0],
            arc.center[1] + radius_vector[1],
            z,
            e,
        ];
        deltas.push([
            current[0] - previous[0],
            current[1] - previous[1],
            current[2] - previous[2],
            current[3] - previous[3],
        ]);
        previous = current;
    }
    let current = [arc.end[0], arc.end[1], arc.end[2], arc.e_delta];
    deltas.push([
        current[0] - previous[0],
        current[1] - previous[1],
        current[2] - previous[2],
        current[3] - previous[3],
    ]);
    deltas
}

fn arc_discretization_steps(radius: f64, angle: f64, deviation: f64) -> usize {
    let distance = radius - deviation;
    if distance < 1.0e-4 {
        if angle < std::f64::consts::PI
            || radius * (1.0 + (std::f64::consts::PI - 0.5 * angle).cos()) < deviation
        {
            1
        } else {
            2
        }
    } else {
        (angle / (2.0 * (distance / radius).acos())).ceil() as usize
    }
}
