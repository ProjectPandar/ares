//! `ShortEdgeCollapse.cpp:11-187` `its_short_edge_collpase` — deterministic
//! mesh decimation used by `SeamPlacer.cpp:650-653` on the occlusion mesh
//! before visibility sampling.

mod shuffle;
mod topology;

use super::mesh::Vec3;
use shuffle::ShuffleRng;
use topology::{face_neighbors, nelson_max_normals};

/// Indexed occlusion mesh: `its_merge` concatenation of per-volume meshes
/// (SeamPlacer.cpp:640-647).
pub(super) struct IndexedMesh {
    pub(super) vertices: Vec<Vec3>,
    pub(super) indices: Vec<[u32; 3]>,
}

/// `ShortEdgeCollapse.cpp:11-187`.
pub(super) fn short_edge_collapse(mesh: &mut IndexedMesh, target_triangle_count: usize) {
    let mut state = CollapseState {
        vertices_index_mapping: (0..mesh.vertices.len()).collect(),
        flatten_queue: Vec::new(),
        triangles_neighbors: face_neighbors(mesh),
        face_removal_flags: vec![false; mesh.indices.len()],
        min_vertex_dot_product: min_vertex_dot_products(mesh),
    };
    let mut face_indices: Vec<usize> = (0..mesh.indices.len()).collect();
    let mut tmp_face_indices = Vec::new();
    let mut decimation_ratio = 1.0_f32;
    let mut edge_len = 0.2_f32;
    let mut generator = ShuffleRng::new(27_644_437);

    while face_indices.len() > target_triangle_count {
        edge_len *= 1.0 + (1.0 - decimation_ratio);
        let mut query = CollapseQuery {
            mesh,
            max_edge_len_squared: edge_len * edge_len,
        };
        generator.shuffle(&mut face_indices);

        let mut allowed_face_removals = face_indices.len() as i64 - target_triangle_count as i64;
        for &face_idx in face_indices.iter() {
            if state.face_removal_flags[face_idx] {
                continue;
            }
            if collapse_shortest_edge(&mut query, &mut state, face_idx) {
                allowed_face_removals -= 2;
            }
            if allowed_face_removals <= 0 {
                break;
            }
        }

        let previous_size = face_indices.len();
        tmp_face_indices.clear();
        tmp_face_indices.extend(
            face_indices
                .iter()
                .copied()
                .filter(|face_idx| !state.face_removal_flags[*face_idx]),
        );
        std::mem::swap(&mut face_indices, &mut tmp_face_indices);
        decimation_ratio = (previous_size - face_indices.len()) as f32 / previous_size as f32;
    }

    // ShortEdgeCollapse.cpp:171-186: extract the decimated mesh.
    let mut final_vertices_mapping: std::collections::HashMap<usize, u32> =
        std::collections::HashMap::new();
    let mut final_vertices = Vec::with_capacity(face_indices.len());
    let mut final_indices = Vec::with_capacity(face_indices.len());
    for idx in face_indices.iter().copied() {
        let mut final_face = [0_u32; 3];
        for (slot, corner) in final_face.iter_mut().enumerate() {
            *corner = state.final_index(mesh.indices[idx][slot] as usize) as u32;
        }
        if final_face[0] == final_face[1]
            || final_face[1] == final_face[2]
            || final_face[2] == final_face[0]
        {
            continue;
        }
        for corner in final_face.iter_mut() {
            let key = *corner as usize;
            let next = final_vertices.len() as u32;
            let mapped = *final_vertices_mapping.entry(key).or_insert(next);
            *corner = mapped;
            if mapped == next {
                final_vertices.push(mesh.vertices[key]);
            }
        }
        final_indices.push(final_face);
    }

    mesh.vertices = final_vertices;
    mesh.indices = final_indices;
}

fn min_vertex_dot_products(mesh: &IndexedMesh) -> Vec<f32> {
    // ShortEdgeCollapse.cpp:41-54: min vertex dot product with
    // `NormalUtils::create_normals` (Nelson-Max weighted) and
    // `its_face_normal` face normals (TriangleMesh.hpp:329-332: the cross
    // uses (b-a, c-b) and the result is normalized twice).
    let vertex_normals = nelson_max_normals(mesh);
    let mut min_vertex_dot_product = vec![1.0_f32; mesh.vertices.len()];
    for face in mesh.indices.iter() {
        let [a, b, c] = [
            mesh.vertices[face[0] as usize],
            mesh.vertices[face[1] as usize],
            mesh.vertices[face[2] as usize],
        ];
        let normal = (b - a).cross(c - b).normalized().normalized();
        for corner in face {
            let corner = *corner as usize;
            let candidate = normal.dot(vertex_normals[corner]);
            if candidate < min_vertex_dot_product[corner] {
                min_vertex_dot_product[corner] = candidate;
            }
        }
    }
    min_vertex_dot_product
}

/// Per-iteration inputs of the collapse pass.
struct CollapseQuery<'a> {
    mesh: &'a IndexedMesh,
    max_edge_len_squared: f32,
}

/// Carry-over collapse state.
struct CollapseState {
    vertices_index_mapping: Vec<usize>,
    flatten_queue: Vec<usize>,
    triangles_neighbors: Vec<[i64; 3]>,
    face_removal_flags: Vec<bool>,
    min_vertex_dot_product: Vec<f32>,
}

impl CollapseState {
    /// ShortEdgeCollapse.cpp:17-31: resolve the merge chain with path
    /// flattening so later lookups are O(1).
    fn final_index(&mut self, orig: usize) -> usize {
        self.flatten_queue.clear();
        let mut idx = orig;
        while self.vertices_index_mapping[idx] != idx {
            self.flatten_queue.push(idx);
            idx = self.vertices_index_mapping[idx];
        }
        for flattened in self.flatten_queue.iter() {
            self.vertices_index_mapping[*flattened] = idx;
        }
        idx
    }
}

/// ShortEdgeCollapse.cpp:105-145: attempt one edge collapse on the given
/// face; returns true when two faces were removed.
fn collapse_shortest_edge(
    query: &mut CollapseQuery<'_>,
    state: &mut CollapseState,
    face_idx: usize,
) -> bool {
    for edge_idx in 0..3 {
        let mut vertex_index_keep =
            state.final_index(query.mesh.indices[face_idx][edge_idx] as usize);
        let mut vertex_index_remove =
            state.final_index(query.mesh.indices[face_idx][(edge_idx + 1) % 3] as usize);
        let distance =
            query.mesh.vertices[vertex_index_keep] - query.mesh.vertices[vertex_index_remove];
        if distance.dot(distance) > query.max_edge_len_squared {
            continue;
        }
        if state.min_vertex_dot_product[vertex_index_remove]
            < state.min_vertex_dot_product[vertex_index_keep]
        {
            std::mem::swap(&mut vertex_index_keep, &mut vertex_index_remove);
        }
        state.vertices_index_mapping[vertex_index_remove] =
            state.vertices_index_mapping[vertex_index_keep];

        let neighbor_to_remove_face_idx = state.triangles_neighbors[face_idx][edge_idx];
        remove_face(face_idx as i64, neighbor_to_remove_face_idx, state);
        remove_face(neighbor_to_remove_face_idx, face_idx as i64, state);
        return true;
    }
    false
}

/// ShortEdgeCollapse.cpp:63-77. Note the `> 0` guards match upstream
/// verbatim (neighbor index 0 is skipped — an upstream quirk kept
/// bug-for-bug).
fn remove_face(face_idx: i64, other_face_idx: i64, state: &mut CollapseState) {
    if face_idx < 0 {
        return;
    }
    let face_idx = face_idx as usize;
    state.face_removal_flags[face_idx] = true;
    let neighbors_of_face = state.triangles_neighbors[face_idx];
    let n_a = if neighbors_of_face[0] != other_face_idx {
        neighbors_of_face[0]
    } else {
        neighbors_of_face[1]
    };
    let n_b = if neighbors_of_face[2] != other_face_idx {
        neighbors_of_face[2]
    } else {
        neighbors_of_face[1]
    };
    if n_a > 0
        && let Some(slot) = state.triangles_neighbors[n_a as usize]
            .iter_mut()
            .position(|slot| *slot == face_idx as i64)
    {
        state.triangles_neighbors[n_a as usize][slot] = n_b;
    }
    if n_b > 0
        && let Some(slot) = state.triangles_neighbors[n_b as usize]
            .iter_mut()
            .position(|slot| *slot == face_idx as i64)
    {
        state.triangles_neighbors[n_b as usize][slot] = n_a;
    }
}
