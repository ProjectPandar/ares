//! Per-layer skirt emission — runs once per chunk before any object
//! content (`GCode.cpp:4388+`), on the layers the skirt covers. Raft
//! chunks emit the skirt around the raft polygons (the skirt layer index
//! is the raft layer index — with a raft, the covered layers start at
//! the raft flange).

use super::super::{motion, skirt};
use super::entry::EntryGeometry;
use super::entry_builder::EntryBuilder;
use super::raft_schedule::RaftSchedule;
use super::schedule::LayerToPrint;
use crate::geometry::ExPolygon;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;
use crate::project_slice::support_material::InstalledSupportLayer;

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    entries: &[LayerToPrint],
    support: &[Vec<InstalledSupportLayer>],
    skirt: &Option<skirt::SkirtPlan>,
    raft: &Option<RaftSchedule>,
    print_z: f64,
    schedule: &mut super::skirt_schedule::SkirtSchedule,
    entry_builder: &mut EntryBuilder<'_>,
    traversal: &PreparedPostClassicTraversal,
    chunk_slices: &[ExPolygon],
    chunk_perimeter_spacing: f64,
) {
    if let (Some(entry), Some(plan)) = (entries.first(), skirt) {
        let object = entry.object_index;
        let Some((skirt_plan_index, skirt_height)) = schedule.next(print_z, plan.layer_count)
        else {
            return;
        };
        let skirt_layer = entry.object_layer.unwrap_or(0);
        let geometry = if let Some(raft_index) = entry.raft_layer {
            let raft_polygons = raft
                .as_ref()
                .and_then(|raft| raft.plans.get(object))
                .and_then(|plans| plans.get(raft_index))
                .map(|plan| {
                    plan.polygons
                        .iter()
                        .map(|polygon| crate::geometry::ExPolygon::new(polygon.clone(), Vec::new()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            EntryGeometry {
                top_surfaces: Vec::new(),
                lower_boundary_lines: Vec::new(),
                nearest_penalties: None,
                layer_slices: raft_polygons.clone().into(),
                internal_surfaces: Vec::new(),
                chunk_slices: raft_polygons,
                chunk_perimeter_spacing: 0.0,
            }
        } else if entry.object_layer.is_some() {
            entry_builder.geometry(object, skirt_layer, chunk_slices, chunk_perimeter_spacing)
        } else {
            let layer = &support[object][entry
                .support_layer
                .expect("support-only pair has a support layer")];
            EntryGeometry {
                top_surfaces: Vec::new(),
                lower_boundary_lines: Vec::new(),
                nearest_penalties: None,
                layer_slices: layer.slices.clone().into(),
                internal_surfaces: Vec::new(),
                chunk_slices: chunk_slices.to_vec(),
                chunk_perimeter_spacing,
            }
        };
        let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
            .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
        let motion_geometry = if entry.object_layer.is_some() {
            geometry.view(traversal, object, skirt_layer, lower_boundary.as_ref())
        } else {
            geometry.view_raft(traversal.scale)
        };
        plan.emit(
            output,
            skirt::SkirtLayer {
                index: skirt_plan_index,
                height_mm: skirt_height,
            },
            motion_geometry,
            state,
        );
    }
}
