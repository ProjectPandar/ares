use super::{Facet, Neighbors};

// admesh/normals.cpp::reverse_facet: local swap and reciprocal encoded slots.
pub(super) fn reverse_facet(facets: &mut [Facet], neighbors: &mut [Neighbors], index: usize) {
    let Neighbors { facet, vnot } = neighbors[index];
    facets[index].vertices.swap(0, 1);
    for (slot, increment) in [3, 4, 2].into_iter().enumerate() {
        let opposite = &mut neighbors[facet[slot]].vnot[(vnot[slot] + 1) % 3];
        *opposite = (*opposite + increment) % 6;
    }
    neighbors[index].facet.swap(1, 2);
    neighbors[index].vnot.swap(1, 2);
    for opposite in &mut neighbors[index].vnot {
        *opposite = (*opposite + 3) % 6;
    }
}

// admesh/normals.cpp::check_normal_vector includes mutation even when fix=false.
pub(super) fn check_normal(facet: &mut Facet, fix: bool) -> bool {
    let normal = calculate(facet);
    if close(normal, facet.normal) {
        facet.normal = normal;
        return false;
    }
    let authored = normalize(facet.normal);
    if close(normal, authored) {
        if fix {
            facet.normal = normal;
        }
        return false;
    }
    let backwards = close(normal, authored.map(|component| -component));
    if fix {
        facet.normal = normal;
    }
    backwards
}

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() < 0.001)
}

// admesh/normals.cpp::stl_fix_normal_directions, including duplicate work items
// and reversal rollback across all previously visited components.
pub(super) fn fix_directions(facets: &mut [Facet], neighbors: &mut [Neighbors]) {
    let mut visited = vec![false; facets.len()];
    let mut reversed = Vec::with_capacity(facets.len());
    let mut pending = Vec::new();
    let mut index = 0;
    if check_normal(&mut facets[index], false) {
        reverse_facet(facets, neighbors, index);
        reversed.push(index);
    }
    visited[index] = true;
    let mut checked = 1;
    'propagation: loop {
        for slot in 0..3 {
            let next = neighbors[index].facet[slot];
            let backwards = neighbors[index].vnot[slot] > 2;
            if backwards && visited[next] {
                break 'propagation;
            }
            if backwards {
                reverse_facet(facets, neighbors, next);
                reversed.push(next);
            }
            if !visited[next] {
                pending.push(next);
            }
        }
        if let Some(next) = pending.pop() {
            index = next;
            if !visited[index] {
                visited[index] = true;
                checked += 1;
            }
        } else {
            if checked == facets.len() {
                return;
            }
            index = visited.iter().position(|seen| !seen).unwrap();
            if check_normal(&mut facets[index], false) {
                reverse_facet(facets, neighbors, index);
                reversed.push(index);
            }
            visited[index] = true;
            checked += 1;
        }
    }
    for previous in reversed.into_iter().rev() {
        reverse_facet(facets, neighbors, previous);
    }
}

pub(super) fn calculate(facet: &Facet) -> [f32; 3] {
    let [a, b, c] = facet.vertices;
    let u = [b.x - a.x, b.y - a.y, b.z - a.z];
    let v = [c.x - a.x, c.y - a.y, c.z - a.z];
    normalize([
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ])
}

pub(super) fn normalize(normal: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = normal.map(f64::from);
    let length = (x * x + (y * y + z * z)).sqrt();
    if length < 1e-12 {
        [0.0; 3]
    } else {
        let reciprocal = (1.0 / length) as f32;
        normal.map(|component| component * reciprocal)
    }
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + (a[1] * b[1] + a[2] * b[2])
}
