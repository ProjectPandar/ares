//! Object entry emission inside the per-chunk object loop — geometry
//! build, labels, `emit_layer`, layer-end timelapse, and the spiral-vase
//! layer accounting (`GCode.cpp:5380-5560`, `SpiralVase.cpp:137-187`).

use super::super::{
    SliceError, motion, object::ObjectLabels, spiral_vase, timelapse as timelapse_core,
};
use super::entry_builder::EntryBuilder;
use super::layer_end;
use crate::geometry::ExPolygon;
use crate::project_slice::island_print_order::OrderedExtrusionLayer;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[expect(
    clippy::too_many_arguments,
    reason = "the object branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    layer: &mut OrderedExtrusionLayer,
    spiral: &mut spiral_vase::SpiralVaseFilter,
    entry_builder: &mut EntryBuilder<'_>,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    chunk_slices: &[ExPolygon],
    chunk_perimeter_spacing: f64,
    labels: &[Option<ObjectLabels>],
    emit_labels: bool,
    traditional_interlude: bool,
    traditional_timelapse: bool,
    timelapse_context: timelapse_core::Context<'_>,
    layer_z: f64,
    layer_height: f64,
    object_layer_counts: &[usize],
    last_entry: Option<(usize, usize)>,
    is_group_end: bool,
    layer_output_start: usize,
) -> Result<(), SliceError> {
    let spiral_body_layer = spiral.is_body_layer(layer, layer_index, f64::from(layer_z));
    state.spiral_vase_layer = spiral_body_layer;
    let geometry = entry_builder.geometry(
        object_index,
        layer_index,
        chunk_slices,
        chunk_perimeter_spacing,
    );
    let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
        .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
    let motion_geometry = geometry.view(
        traversal,
        object_index,
        layer_index,
        lower_boundary.as_ref(),
    );
    let entry_output_start = output.len();
    if let Some(labels) = &labels[object_index] {
        labels.queue_start(output, state, emit_labels);
    }
    let timelapse_inserted =
        motion::emit_layer(output, layer, motion_geometry, state, |output, state| {
            timelapse_core::append_traditional(
                traditional_interlude,
                output,
                state,
                timelapse_context,
            )
        })?;
    let is_last_entry = Some((object_index, layer_index)) == last_entry;
    if let Some(labels) = &labels[object_index] {
        labels.queue_stop(
            output,
            state,
            emit_labels,
            timelapse_inserted && is_group_end,
        );
    }
    if is_group_end {
        layer_end::emit(
            output,
            state,
            labels[object_index].as_ref(),
            timelapse_inserted,
            traditional_timelapse,
            timelapse_context,
        )?;
    }
    spiral.process_layer(
        output,
        spiral_vase::Layer {
            // The upstream filter consumes the FULL layer chunk
            // (change_layer header + Z move + content): the header
            // lines duplicate into the transition_out layer and the
            // chunk's Z-only move is the one rewritten to base_z
            // (`SpiralVase.cpp:137-141`, `:184-187`).
            start: if spiral.covers_chunk() {
                layer_output_start
            } else {
                entry_output_start
            },
            enabled: spiral_body_layer,
            final_layer: is_last_entry && layer_index + 1 == object_layer_counts[object_index],
            z: f64::from(layer_z) + state.options.z_offset,
            height: f64::from(layer_height),
        },
    );
    Ok(())
}
