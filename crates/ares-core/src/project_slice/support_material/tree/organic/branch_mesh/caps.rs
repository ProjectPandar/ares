//! Flat buildplate foot and native hemisphere end caps.

use super::{
    EPS, IndexedTriangleSet, Strip,
    circle::discretize_circle,
    triangulation::{triangulate_fan, triangulate_strip},
};

pub(super) fn bottom(
    mesh: &mut IndexedTriangleSet,
    center: [f64; 3],
    normal: [f64; 3],
    radius: f32,
    flat: bool,
) -> (Strip, f32) {
    if flat {
        let bottom = [center[0] as f32, center[1] as f32, 0.0];
        let top = center.map(|value| value as f32);
        let normal = [0.0, 0.0, 1.0];
        let fan = mesh.vertices.len();
        mesh.vertices.push(bottom);
        let strip = discretize_circle(bottom, normal, radius, EPS, &mut mesh.vertices);
        triangulate_fan::<false>(mesh, fan, strip);
        let top_strip = discretize_circle(top, normal, radius, EPS, &mut mesh.vertices);
        triangulate_strip(mesh, strip, top_strip);
        return (top_strip, 0.0);
    }
    let (steps, angle_step) = hemisphere_steps(radius);
    let fan = mesh.vertices.len();
    let tip = std::array::from_fn(|axis| (center[axis] - normal[axis] * f64::from(radius)) as f32);
    mesh.vertices.push(tip);
    let mut angle = angle_step;
    let mut previous_strip = None;
    for _ in 1..steps {
        let cosine = f64::from(angle.cos());
        let circle_center = std::array::from_fn(|axis| {
            (center[axis] - normal[axis] * f64::from(radius) * cosine) as f32
        });
        let strip = discretize_circle(
            circle_center,
            normal.map(|value| value as f32),
            radius * angle.sin(),
            EPS,
            &mut mesh.vertices,
        );
        if let Some(previous) = previous_strip {
            triangulate_strip(mesh, previous, strip);
        } else {
            triangulate_fan::<false>(mesh, fan, strip);
        }
        previous_strip = Some(strip);
        angle += angle_step;
    }
    (
        previous_strip.expect("branch radius produces a bottom hemisphere ring"),
        tip[2],
    )
}

pub(super) fn top(
    mesh: &mut IndexedTriangleSet,
    center: [f64; 3],
    normal: [f64; 3],
    radius: f32,
    mut previous_strip: Strip,
) -> f32 {
    let (steps, angle_step) = hemisphere_steps(radius);
    let mut angle = std::f64::consts::FRAC_PI_2 as f32;
    for _ in 0..steps {
        let cosine = f64::from(angle.cos());
        let circle_center = std::array::from_fn(|axis| {
            (center[axis] + normal[axis] * f64::from(radius) * cosine) as f32
        });
        let strip = discretize_circle(
            circle_center,
            normal.map(|value| value as f32),
            radius * angle.sin(),
            EPS,
            &mut mesh.vertices,
        );
        triangulate_strip(mesh, previous_strip, strip);
        previous_strip = strip;
        angle -= angle_step;
    }
    let fan = mesh.vertices.len();
    let tip = std::array::from_fn(|axis| (center[axis] + normal[axis] * f64::from(radius)) as f32);
    mesh.vertices.push(tip);
    triangulate_fan::<true>(mesh, fan, previous_strip);
    tip[2]
}

fn hemisphere_steps(radius: f32) -> (usize, f32) {
    let angle_step = (2.0 * (1.0 - f64::from(EPS / radius)).acos()) as f32;
    let steps = (std::f64::consts::PI / (2.0 * f64::from(angle_step))).ceil() as usize;
    (steps, (std::f64::consts::PI / (2.0 * steps as f64)) as f32)
}
