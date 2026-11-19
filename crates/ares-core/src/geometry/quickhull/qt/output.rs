//! Qhull 8.0.2 `poly2_r.c::qh_triangulate` and Orca's ordered mesh collection.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use std::collections::BTreeSet;

use super::{Hull, HullError, Id, Point};
use crate::geometry::ConvexHull3d;

impl Hull {
    pub(super) fn collect_mesh(&self, input: &[[f32; 3]]) -> Result<ConvexHull3d, HullError> {
        let mut mesh = ConvexHull3d {
            vertices: Vec::new(),
            facets: Vec::new(),
            normals: Vec::new(),
        };
        let mut map = vec![u32::MAX; self.vertices.len()];
        // Existing simplicial facets stay in place. The Qt pass appends each
        // polygon's children in ridge order, skipping the two null children
        // at the apex. In dimension three the null-link pass only changes
        // adjacency, which `its_convex_hull` does not read.
        for id in self.suffix(self.first()) {
            let f = &self.facets[id];
            if f.simplicial {
                let indices = f
                    .vertices
                    .as_slice()
                    .try_into()
                    .map_err(|_| HullError::Topology)?;
                self.collect_triangle(indices, f.plane.normal, (&mut mesh, &mut map), input);
            }
        }
        for id in self.suffix(self.first()) {
            let f = &self.facets[id];
            if f.simplicial {
                continue;
            }
            let apex = f.vertices[0];
            let edges = f.ridges.iter().map(|&edge| self.ridges[edge].vertices);
            for [a, b] in edges.filter(|&[a, _]| a != apex) {
                self.collect_triangle([apex, a, b], f.plane.normal, (&mut mesh, &mut map), input);
            }
        }
        // Qt deletes mirror pairs before output. Do not expose a pair of
        // coincident triangles when that repair is required.
        let mut triangles = BTreeSet::new();
        if mesh.facets.iter().any(|&vertices| {
            let mut key = vertices;
            key.sort_unstable();
            key[0] == key[1] || key[1] == key[2] || !triangles.insert(key)
        }) {
            return Err(HullError::Topology);
        }
        Ok(mesh)
    }

    // TriangleMesh.cpp::its_convex_hull. Allocate the vertex indices before
    // fixing their winding, as the order of first encounter is observable.
    fn collect_triangle(
        &self,
        vertices: [Id; 3],
        plane_normal: Point,
        (mesh, map): (&mut ConvexHull3d, &mut [u32]),
        input: &[[f32; 3]],
    ) {
        let mut indices = vertices.map(|id| {
            if map[id] == u32::MAX {
                map[id] = mesh.vertices.len() as u32;
                mesh.vertices.push(input[self.vertices[id]]);
            }
            map[id]
        });
        let [a, b, c] = indices.map(|index| mesh.vertices[index as usize]);
        let u: [f32; 3] = std::array::from_fn(|i| b[i] - a[i]);
        let v: [f32; 3] = std::array::from_fn(|i| c[i] - b[i]);
        let [x, y, z] = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let [nx, ny, nz] = plane_normal;
        if f64::from(x) * nx + f64::from(y) * ny + f64::from(z) * nz < 0.0 {
            indices.swap(1, 2);
        }
        mesh.facets.push(indices);
        mesh.normals.push(plane_normal.map(|c| c as f32));
    }
}
