use super::mesh::{Facet, Mesh};
use crate::geometry::mesh_normals::{dot, sub};
use std::collections::HashMap;

/// `Orient.cpp:281-344`; insertion-order groups and stable ranking per the ARD.
pub(super) fn directions(mesh: &Mesh) -> Vec<[f32; 3]> {
    let mut directions = vec![[0., 0., -1.]];
    add_groups(&mut directions, &mesh.facets, 10);
    add_groups(&mut directions, &mesh.hull, 14);
    const Q: f32 = std::f32::consts::FRAC_1_SQRT_2;
    directions.extend([
        [0., 0., -1.],
        [Q, 0., -Q],
        [0., Q, -Q],
        [-Q, 0., -Q],
        [0., -Q, -Q],
        [1., 0., 0.],
        [Q, Q, 0.],
        [0., 1., 0.],
        [-Q, Q, 0.],
        [-1., 0., 0.],
        [-Q, -Q, 0.],
        [0., -1., 0.],
        [Q, -Q, 0.],
        [Q, 0., Q],
        [0., Q, Q],
        [-Q, 0., Q],
        [0., -Q, Q],
        [0., 0., 1.],
    ]);
    let mut unique = Vec::with_capacity(directions.len());
    for direction in directions {
        if dot(direction, direction) == 0.
            || unique.iter().any(|&prior| {
                let delta = sub(prior, direction);
                dot(delta, delta)
                    <= 1e-7_f32.powi(2) * dot(prior, prior).min(dot(direction, direction))
            })
        {
            continue;
        }
        unique.push(direction);
    }
    unique
        .into_iter()
        .map(|direction| direction.map(|v| -v))
        .collect()
}
fn add_groups(directions: &mut Vec<[f32; 3]>, facets: &[Facet], count: usize) {
    let mut indices = HashMap::new();
    let mut groups: Vec<(f32, f32, [f32; 3])> = Vec::new();
    for facet in facets {
        let key = facet.normal.map(|v| (v * 1000.).floor() as i32);
        let index = *indices.entry(key).or_insert_with(|| {
            groups.push((0., 0., [0.; 3]));
            groups.len() - 1
        });
        let group = &mut groups[index];
        group.0 += facet.area;
        if facet.area > group.1 {
            group.1 = facet.area;
            group.2 = facet.normal;
        }
    }
    groups.sort_by(|a, b| b.0.total_cmp(&a.0));
    directions.extend(groups.into_iter().take(count).map(|group| group.2));
}
