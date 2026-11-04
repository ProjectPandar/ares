//! Object entry emission inside the per-chunk object loop — geometry
//! build, labels, `emit_layer`, layer-end timelapse, and the spiral-vase
//! layer accounting (`GCode.cpp:5380-5560`, `SpiralVase.cpp:137-187`).

use super::super::{
    SliceError, motion, object::ObjectLabels, spiral_vase, timelapse as timelapse_core,
};
use super::entry_builder::EntryBuilder;
use super::layer_end;
use crate::project_slice::island_print_order::{OrderedExtrusionLayer, visits::InstanceVisit};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[expect(
    clippy::too_many_arguments,
    reason = "the object branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    (layer, visit): (&mut OrderedExtrusionLayer, &InstanceVisit),
    spiral: &mut spiral_vase::SpiralVaseFilter,
    entry_builder: &mut EntryBuilder<'_>,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    starts_after_support: bool,
    chunk_perimeter_spacing: f64,
    labels: Option<&ObjectLabels>,
    emit_labels: bool,
    traditional_interlude: bool,
    traditional_timelapse: bool,
    timelapse_context: timelapse_core::Context<'_>,
    layer_z: f64,
    layer_height: f64,
    object_layer_counts: &[usize],
    is_group_end: bool,
    layer_output_start: usize,
) -> Result<(), SliceError> {
    let spiral_body_layer = spiral.is_body_layer(layer, layer_index, layer_z);
    state.spiral_vase_layer = spiral_body_layer;
    let geometry = entry_builder.geometry(object_index, layer_index, chunk_perimeter_spacing);
    let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
        .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
    let motion_geometry = geometry.view(
        timelapse_context,
        object_index,
        layer_index,
        lower_boundary.as_ref(),
    );
    let entry_output_start = output.len();
    if !starts_after_support && let Some(labels) = labels {
        labels.queue_start(output, state, emit_labels);
    }
    let regions = traversal.objects[object_index]
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
    let physical_layer = &traversal.objects[object_index].physical_layers()[layer_index];
    let first_layer =
        physical_layer.id == 0 && (physical_layer.print_z - physical_layer.height).abs() < 0.0001;
    state.options.seam_position = traversal
        .print_object_config(object_index)
        .0
        .object
        .seam_position;
    let reversal = state.reversal_scope(
        traversal,
        object_index,
        motion::SharedLayerId::Object(layer_index),
    );
    let timelapse_inserted = motion::emit_layer(
        output,
        (
            layer,
            visit,
            traversal.objects[object_index].slices(layer_index),
        ),
        (
            motion::EmissionPhase {
                geometry: motion_geometry,
                reversal,
            },
            first_layer,
            |region: usize,
             motion: &mut motion::MotionOptions,
             current_seam_gap: &mut crate::FloatOrPercent| {
                let options = &regions[region].options;
                *current_seam_gap = options.seam_gap;
                // GCode.cpp:6136/6163 `m_config.apply(region.config())`:
                // one region apply refreshes every region-scoped
                // `MotionOptions` field before the phase emits.
                motion.apply_region(
                    &traversal.resolved.views.full.process.region,
                    Some(options),
                    traversal.resolved.views.runtime_gcode.travel_speed.0,
                    motion.nozzle_diameter,
                );
                (
                    options.sparse_infill_density.0,
                    options.is_infill_first.0,
                    super::super::small_area::Gate {
                        enabled: options.small_area_infill_flow_compensation.0,
                        bottom: options.bottom_surface_pattern,
                        internal: options.internal_solid_infill_pattern,
                        top: options.top_surface_pattern,
                    },
                )
            },
        ),
        state,
        |output, state| {
            timelapse_core::append_traditional(
                traditional_interlude,
                output,
                state,
                timelapse_context,
            )
        },
    )?;
    if let Some(labels) = labels {
        labels.queue_stop(output, state, emit_labels);
    }
    if is_group_end {
        layer_end::emit(
            output,
            state,
            layer_end::LayerTimelapse {
                timelapse_inserted,
                traditional_timelapse,
            },
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
            // `GCode.cpp:3805-3806`: `last_layer` is per object batch
            // under the single-object (by-object) pipeline, and the
            // by-layer batch degenerates to the same expression because
            // spiral by-layer prints are admission-limited to one copy
            // (`Print.cpp:1334-1341`).
            final_layer: layer_index + 1 == object_layer_counts[object_index],
            z: layer_z + state.options.z_offset,
            height: layer_height,
        },
    );
    Ok(())
}
