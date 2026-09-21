//! Per-layer skirt emission — runs once per chunk before any object
//! content (`GCode.cpp:4388+`), on the layers the skirt covers. Raft
//! chunks emit the skirt around the raft polygons (the skirt layer index
//! is the raft layer index — with a raft, the covered layers start at
//! the raft flange).

use super::super::{motion, skirt};
use super::entry::EntryGeometry;
use super::entry_builder::EntryBuilder;
use super::raft_schedule::RaftSchedule;
use crate::geometry::ExPolygon;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    entries: &[(usize, usize)],
    skirt: &Option<skirt::SkirtPlan>,
    raft: &Option<RaftSchedule>,
    leading_raft: Option<(usize, usize)>,
    skirt_height: f64,
    entry_builder: &mut EntryBuilder<'_>,
    traversal: &PreparedPostClassicTraversal,
    chunk_slices: &[ExPolygon],
    chunk_perimeter_spacing: f64,
) {
    if let (Some(&(_, skirt_layer)), Some(plan)) = (
        entries.iter().find(|&&(object_index, _)| object_index == 0),
        skirt,
    ) {
        // With a raft, the object layer indices carry the raft
        // offset (`layer->id()` semantics) — the skirt already ran on
        // the raft layers, so a re-run on the object first layer (its
        // object-relative index 0) must not emit again.
        let raft_offset = raft
            .as_ref()
            .map(|raft| raft.counts.first().copied().unwrap_or(0))
            .unwrap_or(0);
        // `plan.emit` returns early when `layer.index >= layer_count`,
        // but the geometry below still indexes the OBJECT layer — keep
        // them separate.
        let skirt_plan_index =
            leading_raft.map_or(skirt_layer + raft_offset, |(raft_index, _)| raft_index);
        let geometry = if let Some((raft_index, _)) = leading_raft {
            let raft_polygons = raft
                .as_ref()
                .and_then(|raft| raft.plans.get(0))
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
                internal_surfaces: &[],
                chunk_slices: raft_polygons,
                chunk_perimeter_spacing: 0.0,
            }
        } else {
            entry_builder.geometry(0, skirt_layer, chunk_slices, chunk_perimeter_spacing)
        };
        let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
            .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
        let motion_geometry = geometry.view(traversal, 0, skirt_layer, lower_boundary.as_ref());
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
