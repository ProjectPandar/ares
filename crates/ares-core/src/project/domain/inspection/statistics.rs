use super::{Bounds, ProjectMesh, ProjectObject, ProjectVolumeType, point_array};
use crate::{SliceError, mesh_slicer::index_mesh_edges};

#[derive(Debug)]
pub struct MeshStatistics {
    pub bounds: Bounds,
    pub facets: usize,
    pub open_edges: usize,
    pub parts: usize,
    pub volume: f32,
}

impl ProjectObject {
    /// Unrepaired model-part statistics, without the instance transformation.
    pub fn mesh_statistics(&self) -> Result<MeshStatistics, SliceError> {
        let mut points = Vec::new();
        let mut facets = 0;
        let mut open_edges = 0;
        let mut parts = 0;
        let mut volume = 0.;
        for part in self
            .volumes()
            .iter()
            .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart)
        {
            let mesh = part.mesh();
            let transform = part.transform();
            points.extend(
                mesh.vertices()
                    .iter()
                    .map(|&p| transform.transform_point_f32(p).map(f64::from)),
            );
            facets += mesh.triangles().len();
            let neighbors = index_mesh_edges(mesh.triangles())?.build_face_neighbors();
            open_edges += neighbors
                .iter()
                .flatten()
                .filter(|edge| edge.is_none())
                .count();
            parts += count_parts(&neighbors);
            volume += (f64::from(mesh_volume(mesh)) * transform.linear_determinant().abs()) as f32;
        }
        Ok(MeshStatistics {
            bounds: Bounds::from_points(points),
            facets,
            open_edges,
            parts,
            volume,
        })
    }
}

fn count_parts(neighbors: &[[Option<usize>; 3]]) -> usize {
    let mut seen = vec![false; neighbors.len()];
    let mut stack = Vec::new();
    let mut parts = 0;
    for first in 0..neighbors.len() {
        if seen[first] {
            continue;
        }
        parts += 1;
        seen[first] = true;
        stack.push(first);
        while let Some(face) = stack.pop() {
            stack.extend(
                neighbors[face]
                    .into_iter()
                    .flatten()
                    .filter(|&adjacent| !std::mem::replace(&mut seen[adjacent], true)),
            );
        }
    }
    parts
}

// TriangleMesh.cpp:1479–1498 uses mesh-local float coordinates and a reference vertex.
fn mesh_volume(mesh: &ProjectMesh) -> f32 {
    let local = |index: u32| point_array(mesh.vertices()[index as usize]).map(|v| v as f32);
    let origin = local(0);
    mesh.triangles()
        .iter()
        .map(|&face| {
            let [a, b, c] = face.map(local);
            let cross = cross(sub(b, a), sub(c, a));
            let magnitude = norm(cross);
            if magnitude == 0. {
                return 0.;
            }
            let normalized = cross.map(|axis| axis / magnitude);
            ((0.5 * magnitude) * dot(normalized, sub(a, origin))) / 3.0
        })
        .sum()
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + (a[1] * b[1] + a[2] * b[2])
}
