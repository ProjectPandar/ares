//! Portable triangulated 3D hull for `TriangleMesh::convex_hull_3d`
//! (`TriangleMesh.cpp:1358-1447`). Predicates use double precision, like Qhull;
//! mesh coordinates and returned facet normals retain upstream float storage.
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHull3d {
    pub vertices: Vec<[f32; 3]>,
    pub facets: Vec<[u32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

struct Face {
    indices: [usize; 3],
    normal: [f64; 3],
    origin: [f64; 3],
    outside: Vec<usize>,
}

impl Face {
    fn new(mut indices: [usize; 3], points: &[[f64; 3]], inside: [f64; 3]) -> Self {
        let origin = points[indices[0]];
        let mut normal = cross(
            sub(points[indices[1]], origin),
            sub(points[indices[2]], origin),
        );
        if dot(normal, sub(inside, origin)) > 0. {
            indices.swap(1, 2);
            normal = normal.map(|value| -value);
        }
        let length = dot(normal, normal).sqrt();
        Self {
            indices,
            normal: normal.map(|value| value / length),
            origin,
            outside: Vec::new(),
        }
    }
    fn distance(&self, point: [f64; 3]) -> f64 {
        dot(self.normal, sub(point, self.origin))
    }
}

/// Deterministic Quickhull. Non-finite points and inputs of affine dimension
/// below three have no three-dimensional hull. Facets have outward winding;
/// coplanar hull polygons are triangulated, not returned as polygon faces.
pub fn convex_hull_3d(input: &[[f32; 3]]) -> Option<ConvexHull3d> {
    if input.len() < 4 || input.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }
    let points: Vec<_> = input.iter().map(|point| point.map(f64::from)).collect();
    let mut extremes = [0; 6];
    for (index, point) in points.iter().enumerate().skip(1) {
        for axis in 0..3 {
            if point[axis] < points[extremes[2 * axis]][axis] {
                extremes[2 * axis] = index;
            }
            if point[axis] > points[extremes[2 * axis + 1]][axis] {
                extremes[2 * axis + 1] = index;
            }
        }
    }
    let mut pair = [0, 0];
    let mut length_squared = 0.;
    for a in extremes {
        for b in extremes {
            let delta = sub(points[b], points[a]);
            let length = dot(delta, delta);
            if length > length_squared {
                pair = [a, b];
                length_squared = length;
            }
        }
    }
    if length_squared == 0. {
        return None;
    }
    let epsilon = length_squared.sqrt() * f64::EPSILON * 64.;
    let [a, b] = pair;
    let line = sub(points[b], points[a]);
    let mut c = a;
    let mut line_distance = 0.;
    for (index, point) in points.iter().enumerate() {
        let n = cross(line, sub(*point, points[a]));
        let distance = dot(n, n) / length_squared;
        if distance > line_distance {
            c = index;
            line_distance = distance;
        }
    }
    if line_distance <= epsilon * epsilon {
        return None;
    }
    let n = cross(line, sub(points[c], points[a]));
    let n_length = dot(n, n).sqrt();
    let mut d = a;
    let mut plane_distance = 0.;
    for (index, point) in points.iter().enumerate() {
        let distance = dot(n, sub(*point, points[a])).abs() / n_length;
        if distance > plane_distance {
            d = index;
            plane_distance = distance;
        }
    }
    if plane_distance <= epsilon {
        return None;
    }
    let inside = std::array::from_fn(|axis| {
        (points[a][axis] + points[b][axis] + points[c][axis] + points[d][axis]) * 0.25
    });
    let mut faces: Vec<_> = [[a, b, c], [a, d, b], [b, d, c], [c, d, a]]
        .map(|indices| Face::new(indices, &points, inside))
        .into();
    for (index, point) in points.iter().enumerate() {
        if ![a, b, c, d].contains(&index) {
            assign(index, *point, &mut faces, epsilon);
        }
    }
    while let Some(face) = faces.iter().find(|face| !face.outside.is_empty()) {
        let mut eye = face.outside[0];
        let mut distance = face.distance(points[eye]);
        for &index in &face.outside[1..] {
            let candidate = face.distance(points[index]);
            if candidate > distance {
                eye = index;
                distance = candidate;
            }
        }
        let mut horizon = Vec::new();
        let mut edges = HashMap::new();
        let mut orphaned = Vec::new();
        let visible = faces.extract_if(.., |face| face.distance(points[eye]) > epsilon);
        for edge in visible.flat_map(|face| {
            orphaned.extend(face.outside);
            let [a, b, c] = face.indices;
            [[a, b], [b, c], [c, a]]
        }) {
            if let Some(index) = edges.remove(&(edge[1], edge[0])) {
                horizon[index] = None;
            } else {
                edges.insert((edge[0], edge[1]), horizon.len());
                horizon.push(Some(edge));
            }
        }
        let first_new = faces.len();
        for [a, b] in horizon.into_iter().flatten() {
            faces.push(Face::new([a, b, eye], &points, inside));
        }
        for index in orphaned {
            if index != eye {
                assign(index, points[index], &mut faces[first_new..], epsilon);
            }
        }
    }
    let mut used = vec![false; points.len()];
    for face in &faces {
        for &index in &face.indices {
            used[index] = true;
        }
    }
    let mut mapping = vec![0; points.len()];
    let mut vertices = Vec::new();
    for (index, point) in input.iter().enumerate() {
        if used[index] {
            mapping[index] = vertices.len() as u32;
            vertices.push(*point);
        }
    }
    let facets: Vec<_> = faces
        .iter()
        .map(|face| face.indices.map(|index| mapping[index]))
        .collect();
    let normals = facets
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|index| vertices[index as usize]);
            super::mesh_normals::face_normal(a, b, c)
        })
        .collect();
    Some(ConvexHull3d {
        vertices,
        facets,
        normals,
    })
}

fn assign(index: usize, point: [f64; 3], faces: &mut [Face], epsilon: f64) {
    let mut best = None;
    let mut distance = epsilon;
    for (index, face) in faces.iter().enumerate() {
        let candidate = face.distance(point);
        if candidate > distance {
            best = Some(index);
            distance = candidate;
        }
    }
    if let Some(face) = best {
        faces[face].outside.push(index);
    }
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
