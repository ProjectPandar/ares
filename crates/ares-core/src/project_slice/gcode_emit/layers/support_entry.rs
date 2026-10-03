//! Native per-instance support dispatch precedes object islands (`GCode.cpp:5390`).

use super::{
    super::{SliceError, motion, object::ObjectLabels, timelapse},
    entry::EntryGeometry,
    entry_builder::EntryBuilder,
    layer_end,
};
use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

#[expect(
    clippy::too_many_arguments,
    reason = "support shares the layer chunk's live writer and labels"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    layer: &mut InstalledSupportLayer,
    selected_role: crate::ExtrusionRole,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    object_layer: Option<usize>,
    entry_builder: &mut EntryBuilder<'_>,
    chunk_slices: &[crate::geometry::ExPolygon],
    chunk_perimeter_spacing: f64,
    labels: &[Option<ObjectLabels>],
    emit_labels: bool,
    standalone: bool,
    is_group_end: bool,
    traditional_timelapse: bool,
    timelapse_context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    let geometry = if let Some(index) = object_layer {
        entry_builder.geometry(object_index, index, chunk_slices, chunk_perimeter_spacing)
    } else {
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
    let mut motion_geometry = if let Some(index) = object_layer {
        geometry.view(traversal, object_index, index, lower_boundary.as_ref())
    } else {
        geometry.view_raft(traversal.scale)
    };
    motion_geometry.internal_surfaces = &[];
    motion_geometry.support_islands = Some(motion::LayerAreas {
        islands: &layer.support_islands,
        tree_base: layer.tree_base_areas.as_deref(),
    });
    state.layer_index = layer.id;
    state.active_layer_print_z = layer.print_z;
    state.on_first_layer = layer.id == 0 && (layer.print_z - layer.height).abs() < 1e-4;
    state.object_layer_over_raft = false;
    state.spiral_vase_layer = false;
    state.avoid_boundary = None;
    if let Some(labels) = &labels[object_index] {
        labels.queue_start(output, state, emit_labels);
    }
    motion::emit_support(
        output,
        &mut layer.support_fills,
        selected_role,
        motion_geometry,
        state,
    );
    if standalone {
        if let Some(labels) = &labels[object_index] {
            labels.queue_stop(output, state, emit_labels, false);
        }
        if is_group_end {
            layer_end::emit(
                output,
                state,
                labels[object_index].as_ref(),
                false,
                traditional_timelapse,
                timelapse_context,
            )?;
        }
    }
    Ok(())
}
