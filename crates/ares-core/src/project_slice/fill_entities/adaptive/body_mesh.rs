use std::collections::HashMap;

use crate::{
    Point3d, ProjectObject, ProjectVolumeType, Transform3d,
    fill::adaptive::octree::{self, OctreeMesh},
    geometry::{CoordinateScale, ExPolygon, Point},
    project_slice::layers::PlannedLayer,
};

use super::body_layout::BodyLayout;

#[derive(Default)]
pub(super) struct Mesh {
    vertices: Vec<[f64; 3]>,
    faces: Vec<[u32; 3]>,
}

impl Mesh {
    pub(super) fn input(&self) -> OctreeMesh<'_> {
        OctreeMesh {
            vertices: &self.vertices,
            triangles: &self.faces,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    // Model.cpp:1706-1740 / PrintObject.cpp:1191-1196. Finish each
    // mesh transform in the indexed triangle set's float vertex storage.
    pub(super) fn rotated(source: &ProjectObject, object: Transform3d) -> Self {
        let mut mesh = Self::default();
        let rotation = octree::octree_rotation_to_octree();
        for volume in source
            .volumes()
            .iter()
            .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        {
            let base = mesh.vertices.len() as u32;
            let transform = volume.transform();
            mesh.vertices
                .extend(volume.mesh().vertices().iter().map(|&v| {
                    let raw = transform.transform_point(v);
                    let raw = Point3d::new(
                        f64::from(raw.x as f32),
                        f64::from(raw.y as f32),
                        f64::from(raw.z as f32),
                    );
                    let p = object.transform_point(raw);
                    octree::rotate_point(rotation, [p.x, p.y, p.z])
                        .map(|value| f64::from(value as f32))
                }));
            let flipped =
                (transform.linear_determinant() < 0.0) != (object.linear_determinant() < 0.0);
            mesh.faces.extend(
                volume
                    .mesh()
                    .triangles()
                    .iter()
                    .map(|triangle| transformed_face(triangle, base, flipped)),
            );
        }
        mesh
    }

    // PrintObject.cpp:1135-1166: sample the connected mesh patches in
    // facet visitation order; each patch is assigned to its best-voted body.
    pub(super) fn split_by_body(
        &self,
        layout: &BodyLayout,
        (layers, islands): (&[PlannedLayer], &[Vec<ExPolygon>]),
        inset: f64,
        scale: CoordinateScale,
    ) -> Option<Vec<Self>> {
        let components = components(&self.faces);
        if components.len() <= 1 {
            return None;
        }
        let mut bodies: Vec<_> = (0..layout.count).map(|_| Self::default()).collect();
        let matrix = octree::octree_rotation_to_octree();
        let to_object = std::array::from_fn(|row| std::array::from_fn(|col| matrix[col][row]));
        for component in components {
            let votes = component_votes(
                &component,
                (&self.faces, &self.vertices),
                layout,
                (layers, islands, inset, scale),
                to_object,
            );
            let (best, count) = votes
                .into_iter()
                .enumerate()
                .rev()
                .max_by_key(|&(_, count)| count)
                .unwrap();
            if count > 0 {
                append_component(&mut bodies[best], self, &component);
            }
        }
        Some(bodies)
    }
}

fn transformed_face(triangle: &[u32; 3], base: u32, flipped: bool) -> [u32; 3] {
    let mut face = triangle.map(|v| v + base);
    if flipped {
        face.swap(1, 2);
    }
    face
}

fn component_votes(
    component: &[usize],
    source: (&[[u32; 3]], &[[f64; 3]]),
    layout: &BodyLayout,
    context: (&[PlannedLayer], &[Vec<ExPolygon>], f64, CoordinateScale),
    to_object: [[f64; 3]; 3],
) -> Vec<usize> {
    let (faces, vertices) = source;
    let mut votes = vec![0; layout.count];
    for &face in component.iter().step_by((component.len() / 8).max(1)) {
        if let Some(body) = sample_body(faces[face], vertices, layout, context, to_object) {
            votes[body] += 1;
        }
    }
    votes
}

fn sample_body(
    face: [u32; 3],
    vertices: &[[f64; 3]],
    layout: &BodyLayout,
    context: (&[PlannedLayer], &[Vec<ExPolygon>], f64, CoordinateScale),
    to_object: [[f64; 3]; 3],
) -> Option<usize> {
    let (layers, islands, inset, scale) = context;
    let [a, b, c] = face.map(|v| vertices[v as usize]);
    let center = octree::rotate_point(to_object, interior_sample(a, b, c, inset));
    let mut layer = layers.partition_point(|layer| layer.slice_z < center[2]);
    if layer == layers.len()
        || (layer > 0 && center[2] - layers[layer - 1].slice_z < layers[layer].slice_z - center[2])
    {
        layer -= 1;
    }
    let point = Point::new(
        scale.checked_scale(center[0]).unwrap(),
        scale.checked_scale(center[1]).unwrap(),
    );
    layout.body_at(layer, &islands[layer], point, scale)
}

fn append_component(mesh: &mut Mesh, source: &Mesh, component: &[usize]) {
    let mut mapped = HashMap::new();
    for &face in component {
        mesh.faces.push(source.faces[face].map(|v| {
            *mapped.entry(v).or_insert_with(|| {
                let ix = mesh.vertices.len() as u32;
                mesh.vertices.push(source.vertices[v as usize]);
                ix
            })
        }));
    }
}

fn interior_sample(a: [f64; 3], b: [f64; 3], c: [f64; 3], inset: f64) -> [f64; 3] {
    let ab: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
    let ac: [f64; 3] = std::array::from_fn(|i| c[i] - a[i]);
    let normal: [f64; 3] = std::array::from_fn(|i| {
        ab[(i + 1) % 3] * ac[(i + 2) % 3] - ab[(i + 2) % 3] * ac[(i + 1) % 3]
    });
    let area = normal.iter().map(|n| n * n).sum::<f64>().sqrt();
    std::array::from_fn(|i| {
        (a[i] + b[i] + c[i]) / 3.0
            - if area > 0.0 {
                normal[i] * (inset / area)
            } else {
                0.0
            }
    })
}

// MeshSplitImpl.hpp:48-97,143-190: visit edge-connected facets using
// the first unvisited facet as seed and a last-in, first-out work stack.
fn components(faces: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut neighbors = vec![[None; 3]; faces.len()];
    let mut unmatched: HashMap<(u32, u32), (usize, usize)> = HashMap::with_capacity(faces.len());
    for (face, &[a, b, c]) in faces.iter().enumerate() {
        for (edge, (v, w)) in [(a, b), (b, c), (c, a)].into_iter().enumerate() {
            if let Some((other, back_edge)) = unmatched.remove(&(w, v)) {
                neighbors[face][edge] = Some(other);
                neighbors[other][back_edge] = Some(face);
            } else {
                unmatched.insert((v, w), (face, edge));
            }
        }
    }
    let mut visited = vec![false; faces.len()];
    let mut stack = Vec::new();
    let mut result = Vec::new();
    for seed in 0..faces.len() {
        if visited[seed] {
            continue;
        }
        visited[seed] = true;
        let mut component = vec![seed];
        stack.push(seed);
        while let Some(face) = stack.pop() {
            let next: Vec<_> = neighbors[face]
                .into_iter()
                .flatten()
                .filter(|&neighbor| !visited[neighbor])
                .collect();
            for neighbor in next {
                visited[neighbor] = true;
                component.push(neighbor);
                stack.push(neighbor);
            }
        }
        result.push(component);
    }
    result
}
