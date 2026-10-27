//! Per-entry `EntryGeometry` builder — the traversal-lookup closure from
//! `layers::append`, hoisted into a struct so the borrow set can be passed
//! to the group emit helpers as one `&mut` argument.

use super::entry::EntryGeometry;
use crate::geometry::ExPolygon;
use crate::project_slice::extrusion_islands::PreparedPostExtrusionIslands;
use crate::project_slice::gcode_emit::motion::CurledLine;
use crate::project_slice::island_print_order::{self, NearestSeamLayer};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

pub(super) struct EntryBuilder<'a> {
    traversal: &'a PreparedPostClassicTraversal,
    top_surfaces: &'a [Vec<Vec<ExPolygon>>],
    nearest_seam_plans: &'a [Vec<NearestSeamLayer>],
    curled_lines: &'a [Vec<Vec<CurledLine>>],
    island_predecessor: &'a PreparedPostExtrusionIslands,
    layer_boundary_cache: std::collections::HashMap<(usize, usize), std::rc::Rc<[ExPolygon]>>,
    /// The chunk's EXTERNAL-boundary slices, instance-translated
    /// (`get_boundary_external`, `AvoidCrossingPerimeters.cpp:1172-1177`),
    /// installed once per chunk by `layers/drive.rs`.
    pub(super) boundary_slices: Vec<ExPolygon>,
}

impl<'a> EntryBuilder<'a> {
    pub(super) fn new(
        traversal: &'a PreparedPostClassicTraversal,
        top_surfaces: &'a [Vec<Vec<ExPolygon>>],
        curled_lines: &'a [Vec<Vec<CurledLine>>],
        nearest_seam_plans: &'a [Vec<NearestSeamLayer>],
        island_predecessor: &'a PreparedPostExtrusionIslands,
    ) -> Self {
        Self {
            traversal,
            top_surfaces,
            curled_lines,
            nearest_seam_plans,
            island_predecessor,
            layer_boundary_cache: std::collections::HashMap::new(),
            boundary_slices: Vec::new(),
        }
    }

    pub(super) fn geometry(
        &mut self,
        object_index: usize,
        layer_index: usize,
        chunk_perimeter_spacing: f64,
    ) -> EntryGeometry<'a> {
        let lower_boundary_lines = self.traversal.objects[object_index]
            .lower_slices(layer_index)
            .into_iter()
            .flatten()
            .flat_map(ExPolygon::lines)
            .collect::<Vec<_>>();
        EntryGeometry {
            top_surfaces: self.top_surfaces[object_index]
                .get(layer_index)
                .map(|expolygons| expolygons.iter().collect::<Vec<_>>())
                .unwrap_or_default(),
            lower_boundary_lines,
            nearest_penalties: self
                .nearest_seam_plans
                .get(object_index)
                .and_then(|plans| plans.get(layer_index)),
            previous_curled: self
                .curled_lines
                .get(object_index)
                .and_then(|layers| layer_index.checked_sub(1).and_then(|prev| layers.get(prev)))
                .map_or(&[], |lines| lines),
            layer_slices: super::super::layer_boundary_slices_rc(
                self.traversal,
                object_index,
                layer_index,
                &mut self.layer_boundary_cache,
            ),
            internal_surfaces: island_print_order::internal_surfaces(
                self.island_predecessor,
                object_index,
                layer_index,
            ),
            chunk_slices: self.boundary_slices.clone(),
            chunk_perimeter_spacing,
        }
    }
}
