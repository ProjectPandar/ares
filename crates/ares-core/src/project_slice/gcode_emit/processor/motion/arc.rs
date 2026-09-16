use super::word;
use crate::options::GCodeFlavor;

/// `Geometry::ArcWelder::arc_center` (`ArcWelder.hpp:23-42`): the center of
/// an arc through `start`/`end` with signed radius `r`; positive radius and
/// CCW pick one side, the other combination the mirror. Returns None when
/// the endpoints coincide.
pub(in crate::project_slice::gcode_emit::processor) fn center_from_radius(
    start: [f64; 2],
    end: [f64; 2],
    radius: f64,
    ccw: bool,
) -> Option<[f64; 2]> {
    let v = [end[0] - start[0], end[1] - start[1]];
    let q2 = v[0] * v[0] + v[1] * v[1];
    if q2 <= 0.0 {
        return None;
    }
    let t2 = radius * radius / q2 - 0.25;
    let t = if t2 > 0.0 { t2.sqrt() } else { 0.0 };
    let mid = [0.5 * (start[0] + end[0]), 0.5 * (start[1] + end[1])];
    let vp = [-v[1] * t, v[0] * t];
    Some(if (radius > 0.0) == ccw {
        [mid[0] + vp[0], mid[1] + vp[1]]
    } else {
        [mid[0] - vp[0], mid[1] - vp[1]]
    })
}

pub(super) struct ArcMotion {
    pub(super) start: [f64; 3],
    pub(super) end: [f64; 3],
    pub(super) e_delta: f64,
    pub(super) feedrate: f64,
    pub(super) gcode_flavor: GCodeFlavor,
}

/// The shared arc geometry (`process_G2_G3`, `GCodeProcessor.cpp:4548-
/// 4670`): the f32 center (`Vec3f rel_center`), the f64 start radius
/// (Eigen `.norm()` = the naive sqrt of the squared sum), and the signed
/// sweep — full circle (`|delta| < 1e-4`) pins 2*pi before the
/// clockwise adjustment. ONE derivation for the block generator, the id
/// counter, and the whole-arc first block.
pub(in crate::project_slice::gcode_emit::processor) struct ParsedArc {
    pub(in crate::project_slice::gcode_emit::processor) center: [f64; 2],
    pub(in crate::project_slice::gcode_emit::processor) start_radius: [f64; 2],
    pub(in crate::project_slice::gcode_emit::processor) radius: f64,
    pub(in crate::project_slice::gcode_emit::processor) sweep: f64,
}

pub(in crate::project_slice::gcode_emit::processor) fn parse_arc(
    command: &str,
    code: &str,
    start: [f64; 3],
    end: [f64; 3],
) -> Option<ParsedArc> {
    let (i, j) = match word(code, 'R').filter(|r| *r != 0.0) {
        Some(r) => {
            let center =
                center_from_radius([start[0], start[1]], [end[0], end[1]], r, command != "G2")?;
            ((center[0] - start[0]) as f32, (center[1] - start[1]) as f32)
        }
        None => (
            word(code, 'I').unwrap_or(0.0) as f32,
            word(code, 'J').unwrap_or(0.0) as f32,
        ),
    };
    let center = [start[0] + f64::from(i), start[1] + f64::from(j)];
    let start_radius = [start[0] - center[0], start[1] - center[1]];
    let end_radius = [end[0] - center[0], end[1] - center[1]];
    // `Arc::start_radius()` is Eigen's `.norm()` = the naive sqrt of the
    // squared sum — not the compensated `hypot` — and the discretization
    // ceil sees the exact double it produces.
    let radius = (start_radius[0] * start_radius[0] + start_radius[1] * start_radius[1]).sqrt();
    if radius <= f64::EPSILON {
        return None;
    }
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
    Some(ParsedArc {
        center,
        start_radius,
        radius,
        sweep,
    })
}

/// `ArcWelder::arc_discretization_steps` (`ArcWelder.hpp:48-64`) — the
/// non-MarlinFirmware segment count at the 0.0125mm gcode tolerance.
pub(in crate::project_slice::gcode_emit::processor) fn arc_discretization_steps(
    radius: f64,
    angle: f64,
    deviation: f64,
) -> usize {
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

pub(super) fn deltas(command: &str, code: &str, motion: ArcMotion) -> Option<Vec<[f64; 4]>> {
    let ArcMotion {
        start,
        end,
        e_delta,
        feedrate,
        gcode_flavor,
    } = motion;
    let parsed = parse_arc(command, code, start, end)?;
    let ParsedArc {
        center,
        start_radius,
        radius: radius_f64,
        sweep,
    } = parsed;
    let radius = radius_f64 as f32;
    let i = (center[0] - start[0]) as f32;
    let j = (center[1] - start[1]) as f32;
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
    // `radius_mm = rel_center.norm()` (`GCodeProcessor.cpp:4723`): the
    // Marlin segment math runs on the Vec3f (f32) norm of the I/J
    // offsets — NOT the Vec3d `start_radius()` the legacy branch uses.
    // Keeping the two derivations separate preserves boundary-exact
    // segment counts (verified at the F458.1401 boundary where the
    // f64 radius flips the ceil by one).
    let radius_mm = (arc.i * arc.i + arc.j * arc.j).sqrt();
    let segment_mm = (8.0_f32 * radius_mm * 0.02)
        .sqrt()
        .min(arc.feedrate as f32 * (1.0 / 50.0))
        .clamp(0.1, 2.0);
    let flat_mm = (f64::from(radius_mm) * arc.sweep.abs()) as f32;
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
    // `Arc::start_radius()` is Eigen's `.norm()` = the naive sqrt of the
    // squared sum — not the compensated `hypot` — and the discretization
    // ceil sees the exact double it produces.
    let radius = (arc.start_radius[0] * arc.start_radius[0]
        + arc.start_radius[1] * arc.start_radius[1])
        .sqrt();
    let segments = arc_discretization_steps(radius, arc.sweep.abs(), 0.0125);
    if let Ok(path) = std::env::var("ARES_DUMP_ARCS") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "ARC n={segments} r={radius:.9} a={:.9}",
                arc.sweep.abs()
            );
        }
    }
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
        // The processor's position slots are `AxisCoords` (float), so the
        // per-segment delta upstream is float(end) - float(start)
        // (`GCodeProcessor.cpp:3900` delta_pos arithmetic): segments whose
        // delta vanishes in f32 land as Noop and never become blocks.
        let delta32 = |current: f64, previous: f64| (current as f32 - previous as f32) as f64;
        deltas.push([
            delta32(current[0], previous[0]),
            delta32(current[1], previous[1]),
            delta32(current[2], previous[2]),
            delta32(current[3], previous[3]),
        ]);
        previous = current;
    }
    let current = [arc.end[0], arc.end[1], arc.end[2], arc.e_delta];
    let delta32 = |current: f64, previous: f64| (current as f32 - previous as f32) as f64;
    deltas.push([
        delta32(current[0], previous[0]),
        delta32(current[1], previous[1]),
        delta32(current[2], previous[2]),
        delta32(current[3], previous[3]),
    ]);
    deltas
}
