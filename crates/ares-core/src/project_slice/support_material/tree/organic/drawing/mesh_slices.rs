//! Branch tube slicing, safety trimming and layer-range alignment.

use super::super::{branch_mesh::IndexedTriangleSet, interface_placer::layer_z, layer_index};
use super::{Branch, BranchSlices, Drawing, clipping, geometry_error};
use crate::{
    SliceError,
    geometry::intersection_polygons_paths_with_safety_offset,
    mesh_slicer::{
        RawIntersectionBudget, SlicingMode, apply_slicing_mode,
        chain_lines_by_triangle_connectivity, make_loops, slice_mesh_on_planes,
    },
};

impl Drawing<'_, '_> {
    pub(super) fn slice_branch(
        &mut self,
        branch: &Branch,
        mesh: &mut IndexedTriangleSet,
        slice_z: &mut Vec<f32>,
    ) -> Result<Option<BranchSlices>, SliceError> {
        mesh.vertices.clear();
        mesh.indices.clear();
        let path: Vec<_> = branch.path.iter().map(|&node| self.element(node)).collect();
        let (zmin, zmax) = self.extrusion.extrude_branch(&path, branch.has_root, mesh);
        let config = self.config();
        let slicing = self.extrusion.slicing;
        let first = path[0].state.layer_index;
        let last = path[path.len() - 1].state.layer_index;
        let mut begin = if branch.has_root {
            first
        } else {
            first.min(layer_index::ceil(slicing, config, f64::from(zmin)))
        };
        let end = if branch.has_tip {
            last
        } else {
            last.max(layer_index::floor(slicing, config, f64::from(zmax)))
        } + 1;
        slice_z.clear();
        for layer in begin..end {
            let print_z = layer_z(slicing, config, layer);
            let bottom_z = if layer > 0 {
                layer_z(slicing, config, layer - 1)
            } else {
                0.0
            };
            slice_z.push((0.5 * (bottom_z + print_z)) as f32);
        }
        let topology = crate::mesh_slicer::index_mesh_edges(&mesh.indices)?;
        let plane_vertices = mesh
            .vertices
            .iter()
            .map(|p| p.map(f64::from))
            .collect::<Vec<_>>();
        let coplanar = crate::mesh_slicer::coplanar_edges(
            &plane_vertices,
            &mesh.indices,
            &topology,
            self.extrusion.scale.factor(),
        );
        let scale = self.extrusion.scale;
        // Native slice_mesh scales XY in float before facet intersection.
        for vertex in &mut mesh.vertices {
            if slice_z.len() <= 1 {
                vertex[0] /= scale.factor() as f32;
                vertex[1] /= scale.factor() as f32;
            } else {
                let multiplier = (1.0 / scale.factor()) as f32;
                vertex[0] *= multiplier;
                vertex[1] *= multiplier;
            }
        }
        let raw = slice_mesh_on_planes(
            &mesh.vertices,
            &mesh.indices,
            slice_z,
            &mut RawIntersectionBudget::new(),
        )?;
        let mut polygons = Vec::with_capacity(raw.len());
        for (offset, lines) in raw.into_iter().enumerate() {
            let mut loops = make_loops(
                chain_lines_by_triangle_connectivity(lines, &coplanar),
                scale.scaled_delta(2.0) as i64,
            );
            apply_slicing_mode(&mut loops, SlicingMode::Positive);
            let collision = self
                .volumes
                .collision(0, begin + offset, true)
                .map_err(geometry_error)?;
            let clipped =
                clipping::difference(loops.polygons(), collision, scale).map_err(geometry_error)?;
            let mut clipped = intersection_polygons_paths_with_safety_offset(
                &clipped,
                std::slice::from_ref(&self.volumes.bed_area),
            )
            .map_err(geometry_error)?;
            clipping::remove_small(&mut clipped, scale);
            polygons.push(clipped);
        }
        // Both empty output and all-empty leading slices are skipped upstream.
        let Some(leading) = polygons.iter().position(|slice| !slice.is_empty()) else {
            return Ok(None);
        };
        polygons.drain(..leading);
        begin += leading;
        let mut slices = BranchSlices {
            begin,
            polygons,
            contacts: Vec::new(),
        };
        if branch.has_root {
            self.root_contacts(branch, &mut slices)
                .map_err(geometry_error)?;
        }
        Ok(Some(slices))
    }
}
