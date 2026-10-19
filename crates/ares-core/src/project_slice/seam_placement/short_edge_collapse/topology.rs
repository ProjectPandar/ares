//! Mesh adjacency and vertex-normal inputs to `its_short_edge_collpase`.

use super::IndexedMesh;
use crate::project_slice::seam_placement::mesh::Vec3;

/// `create_face_neighbors_index` sequential semantics
/// (MeshSplitImpl.hpp:224-295): for each unassigned edge, the first later
/// face sharing the oppositely-oriented edge wins.
pub(super) fn face_neighbors(mesh: &IndexedMesh) -> Vec<[i64; 3]> {
    let no_value: i64 = -1;
    let mut vertex_triangles: Vec<Vec<u32>> = vec![Vec::new(); mesh.vertices.len()];
    for (face_idx, face) in mesh.indices.iter().enumerate() {
        for corner in face {
            vertex_triangles[*corner as usize].push(face_idx as u32);
        }
    }
    let mut neighbors = vec![[no_value; 3]; mesh.indices.len()];
    {
        let mut search = EdgeNeighborSearch {
            mesh,
            vertex_triangles: &vertex_triangles,
            neighbors: &mut neighbors,
        };
        for face_idx in 0..mesh.indices.len() {
            let triangle_indices = mesh.indices[face_idx];
            for edge_index in 0..3 {
                search.assign(face_idx, triangle_indices, edge_index);
            }
        }
    }
    neighbors
}

/// `its_triangle_edge` + the oppositely-oriented shared-edge search
/// (MeshSplitImpl.hpp:248-271): the first later face containing the second
/// edge vertex, oriented backwards, claims the edge.
struct EdgeNeighborSearch<'a> {
    mesh: &'a IndexedMesh,
    vertex_triangles: &'a [Vec<u32>],
    neighbors: &'a mut Vec<[i64; 3]>,
}

impl EdgeNeighborSearch<'_> {
    fn assign(&mut self, face_idx: usize, triangle_indices: [u32; 3], edge_index: usize) {
        let no_value: i64 = -1;
        if self.neighbors[face_idx][edge_index] != no_value {
            // This edge already has a neighbor assigned
            // (MeshSplitImpl.hpp:244-247).
            return;
        }
        let edge_first = triangle_indices[edge_index];
        let edge_second = triangle_indices[(edge_index + 1) % 3];
        for other_face in self.vertex_triangles[edge_first as usize].iter().copied() {
            if other_face as usize <= face_idx {
                continue;
            }
            let face_indices = self.mesh.indices[other_face as usize];
            let Some(vertex_index) = its_triangle_vertex_index(face_indices, edge_second) else {
                continue;
            };
            if edge_first != face_indices[(vertex_index + 1) % 3] {
                continue;
            }
            if self.neighbors[other_face as usize][vertex_index] != no_value {
                continue;
            }
            self.neighbors[face_idx][edge_index] = other_face as i64;
            self.neighbors[other_face as usize][vertex_index] = face_idx as i64;
            break;
        }
    }
}

/// `its_triangle_vertex_index` (TriangleMesh.hpp:247-252).
fn its_triangle_vertex_index(face: [u32; 3], vertex_idx: u32) -> Option<usize> {
    if face[0] == vertex_idx {
        Some(0)
    } else if face[1] == vertex_idx {
        Some(1)
    } else if face[2] == vertex_idx {
        Some(2)
    } else {
        None
    }
}

/// `NormalUtils::create_normals_nelson_weighted` (NormalUtils.cpp:96-127;
/// the default `VertexNormalType` at NormalUtils.hpp:51).
pub(super) fn nelson_max_normals(mesh: &IndexedMesh) -> Vec<Vec3> {
    let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); mesh.vertices.len()];
    let mut count = vec![0.0_f32; mesh.vertices.len()];
    for face in mesh.indices.iter() {
        let [a, b, c] = [
            mesh.vertices[face[0] as usize],
            mesh.vertices[face[1] as usize],
            mesh.vertices[face[2] as usize],
        ];
        let normal = (b - a).cross(c - a).normalized();
        let e0 = (a - b).norm();
        let e1 = (b - c).norm();
        let e2 = (c - a).norm();
        let coefficients = [e0 * e2, e0 * e1, e1 * e2];
        for (corner, weight) in face.iter().zip(coefficients) {
            let corner = *corner as usize;
            normals[corner] += normal * weight;
            count[corner] += weight;
        }
    }
    for (normal, weight) in normals.iter_mut().zip(count) {
        *normal = *normal / weight;
    }
    normals
}
