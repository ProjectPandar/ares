//! Per-entry `EntryGeometry` builder — the traversal-lookup closure from
//! `layers::append`, hoisted into a struct so the borrow set can be passed
//! to the group emit helpers as one `&mut` argument.

use super::{entry::EntryGeometry, schedule::LayerToPrint};
use crate::geometry::ExPolygon;
use crate::project_slice::extrusion_islands::PreparedPostExtrusionIslands;
use crate::project_slice::gcode_emit::motion::CurledLine;
use crate::project_slice::island_print_order::{self, NearestSeamLayer, OrderedExtrusionLayer};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[derive(Clone, Default)]
struct EstimatorHistory {
    previous: Option<usize>,
    last_prepared: Option<usize>,
}

pub(super) struct EntryBuilder<'a> {
    traversal: &'a PreparedPostClassicTraversal,
    top_surfaces: &'a [Vec<Vec<ExPolygon>>],
    nearest_seam_plans: &'a [Vec<NearestSeamLayer>],
    curled_lines: &'a [Vec<Vec<CurledLine>>],
    island_predecessor: &'a PreparedPostExtrusionIslands,
    estimator_history: Vec<EstimatorHistory>,
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
            estimator_history: vec![EstimatorHistory::default(); traversal.objects.len()],
            layer_boundary_cache: std::collections::HashMap::new(),
            boundary_slices: Vec::new(),
        }
    }

    /// GCode.cpp:5102–5108,6995–7001: only a printed object layer with
    /// an enabled overhang estimator advances the history. Support-only
    /// chunks and empty geometric slices must not become its reference.
    pub(super) fn prepare_overhang(
        &mut self,
        entries: &[LayerToPrint],
        objects: &[Vec<OrderedExtrusionLayer>],
        nozzle: usize,
    ) {
        let overhang_fan = self
            .traversal
            .resolved
            .views
            .full
            .filament
            .print
            .enable_overhang_bridge_fan
            .0
            .iter()
            .any(|enabled| enabled.0);
        for entry in entries {
            let Some(layer_index) = entry.object_layer else {
                continue;
            };
            let object_index = entry.object_index;
            let regions = self.traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .as_parts()
                .0
                .as_parts()
                .0
                .as_parts()
                .2;
            let prepares = objects[object_index][layer_index]
                .tool_regions
                .iter()
                .zip(regions)
                .any(|(usage, region)| {
                    (usage.perimeters
                        || usage.outer_wall_fill
                        || usage.internal_solid
                        || usage.top
                        || usage.bottom
                        || usage.infill)
                        && (overhang_fan || region.options.enable_overhang_speed.get_at(nozzle))
                });
            if prepares {
                let history = &mut self.estimator_history[object_index];
                history.previous = history.last_prepared.replace(layer_index);
            }
        }
    }

    pub(super) fn geometry(
        &mut self,
        object_index: usize,
        layer_index: usize,
        chunk_perimeter_spacing: f64,
    ) -> EntryGeometry<'a> {
        let previous = self.estimator_history[object_index].previous;
        let lower_boundary_lines = previous
            .and_then(|layer| self.traversal.objects[object_index].slices(layer))
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
                .and_then(|layers| previous.and_then(|index| layers.get(index)))
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
