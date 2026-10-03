//! `TreeSupport3D.cpp::discretize_circle`.

use super::{
    Strip,
    vector::{cross, normalized_f32},
};

pub(super) fn discretize_circle(
    center: [f32; 3],
    normal: [f32; 3],
    radius: f32,
    eps: f32,
    points: &mut Vec<[f32; 3]>,
) -> Strip {
    let mut angle_step = (2.0 * (1.0 - f64::from(eps / radius)).acos()) as f32;
    let steps = (std::f64::consts::TAU / f64::from(angle_step)).ceil() as usize;
    angle_step = (std::f64::consts::TAU / steps as f64) as f32;
    let x = normalized_f32(cross(normal, [0.0, -1.0, 0.0]));
    let y = normalized_f32(cross(normal, x));
    let x = x.map(|value| value * radius);
    let y = y.map(|value| value * radius);
    let begin = points.len();
    points.reserve(steps);
    let mut angle = 0.0_f32;
    for _ in 0..steps {
        let cosine = angle.cos();
        let sine = angle.sin();
        points.push(std::array::from_fn(|axis| {
            (center[axis] + x[axis] * cosine) + y[axis] * sine
        }));
        angle += angle_step;
    }
    (begin, points.len())
}
