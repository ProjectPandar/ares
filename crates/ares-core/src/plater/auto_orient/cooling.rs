//! `Orient.cpp:512-571`; the class cooling search, not `orient_for_cooling`.
use super::mesh::Mesh;
use crate::geometry::mesh_normals::{dot, face_normal};

pub(super) fn direction(
    mesh: &Mesh,
    euler: [f64; 3],
    weights: &[f32],
    machine: [f64; 3],
) -> [f64; 3] {
    let mut facets: Vec<_> = mesh.facets.iter().map(|f| f.vertices).collect();
    for (axis, angle) in euler.into_iter().enumerate() {
        let angle = angle as f32;
        if angle == 0. {
            continue;
        }
        let degrees = 180_f32 * angle / std::f32::consts::PI;
        let radians = (f64::from(degrees) / 180.) * std::f64::consts::PI;
        let (s, c) = (radians.sin(), radians.cos());
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        for point in facets.iter_mut().flatten() {
            let x = f64::from(point[u]);
            let y = f64::from(point[v]);
            point[u] = (x * c - y * s) as f32;
            point[v] = (x * s + y * c) as f32;
        }
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in facets.iter().flatten() {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    let shift = [
        -((f64::from(min[0]) + f64::from(max[0])) * 0.5) as f32,
        -((f64::from(min[1]) + f64::from(max[1])) * 0.5) as f32,
        -min[2],
    ];
    for point in facets.iter_mut().flatten() {
        for axis in 0..3 {
            point[axis] += shift[axis];
        }
    }
    let normals: Vec<_> = facets
        .into_iter()
        .map(|[a, b, c]| face_normal(a, b, c))
        .collect();
    let machine = machine.map(|v| v as f32);
    let mut best = [0.; 3];
    let mut maximum = 0_f32;
    let mut origin = 0_f32;
    for sample in 0..180 {
        let degrees = (sample * 2) as f32;
        let angle = (f64::from(degrees) * (std::f64::consts::PI / 180.)) as f32;
        let direction = [angle.cos(), angle.sin(), 0.];
        let mut projected = 0.;
        for (normal, &weight) in normals.iter().zip(weights) {
            let projection = dot(*normal, direction);
            if weight > 0. && projection > 0. {
                projected += weight * projection;
            }
        }
        if projected > maximum {
            maximum = projected;
            best = direction;
        }
        if dot(direction, machine) > 0.999 {
            origin = projected;
        }
    }
    if (origin - maximum).abs() < 1. {
        best = machine;
    }
    best.map(f64::from)
}
