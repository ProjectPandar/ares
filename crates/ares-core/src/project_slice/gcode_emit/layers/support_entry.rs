//! Native per-instance support dispatch precedes object islands (`GCode.cpp:5390`).

#[cfg(test)]
#[path = "support_entry/default_spacing_tests.rs"]
mod default_spacing_tests;

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
    labels: Option<&ObjectLabels>,
    emit_labels: bool,
    standalone: bool,
    is_group_end: bool,
    traditional_timelapse: bool,
    timelapse_context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    let geometry = if let Some(index) = object_layer {
        entry_builder.geometry(object_index, index, chunk_perimeter_spacing)
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
    motion_geometry.avoid_crossing.top_surfaces = &[];
    let (source_object_index, _) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let resolved = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == source_object_index)
        .unwrap();
    let source = &traversal.project.objects()[source_object_index];
    let has_brim = traversal
        .resolved
        .objects
        .iter()
        .any(|object| crate::project::effective_config::usage::supported_brim(&object.object));
    let spacing =
        default_perimeter_spacing(&traversal.resolved.views.full, resolved, source, has_brim);
    let scale_ratio = motion_geometry.scale.checked_scale(1.0).unwrap() as f64 / 1_000_000.0;
    motion_geometry.support_islands = Some(motion::LayerAreas {
        scaled_default_perimeter_spacing: Some((f64::from(spacing) * scale_ratio) as f32),
        islands: &layer.support_islands,
        tree_base: layer.tree_base_areas.as_deref(),
    });
    state.layer_index = layer.id;
    state.active_layer_print_z = layer.print_z;
    state.on_first_layer = layer.id == 0 && (layer.print_z - layer.height).abs() < 1e-4;
    state.object_layer_over_raft = false;
    state.spiral_vase_layer = false;
    state.avoid_boundary = None;
    if let Some(labels) = labels {
        labels.queue_start(output, state, emit_labels);
    }
    let phase = motion::EmissionPhase {
        geometry: motion_geometry,
        reversal: state.reversal_scope(
            traversal,
            object_index,
            motion::SharedLayerId::Support(layer.id),
        ),
    };
    motion::emit_support(
        output,
        &mut layer.support_fills,
        selected_role,
        phase,
        state,
    )?;
    if standalone {
        if let Some(labels) = labels {
            labels.queue_stop(output, state, emit_labels);
        }
        if is_group_end {
            layer_end::emit(
                output,
                state,
                layer_end::LayerTimelapse {
                    timelapse_inserted: false,
                    traditional_timelapse,
                },
                timelapse_context,
            )?;
        }
    }
    Ok(())
}

// AvoidCrossingPerimeters.cpp:480-488 and PrintObject.cpp:3784-3803.
fn default_perimeter_spacing(
    settings: &crate::ProjectSettings,
    resolved: &crate::project::effective_config::types::ResolvedProjectObject,
    source: &crate::ProjectObject,
    has_brim: bool,
) -> f32 {
    let mut extruders = Vec::with_capacity(resolved.all_regions.len() * 3 + source.volumes().len());
    for region in &resolved.all_regions {
        crate::project::effective_config::usage::append_region_filaments(
            &mut extruders,
            region,
            has_brim,
        );
    }
    for volume in source.volumes().iter().filter(|volume| {
        matches!(
            volume.volume_type(),
            crate::ProjectVolumeType::ModelPart | crate::ProjectVolumeType::ParameterModifier
        )
    }) {
        let selector = match volume.region_overrides().extruder {
            Some(selector) if selector.0 != 0 => selector.0,
            _ => source
                .region_overrides()
                .extruder
                .map_or(1, |value| value.0),
        };
        if selector > 0 {
            extruders.push(selector as usize - 1);
        }
    }
    extruders.sort_unstable();
    extruders.dedup();
    let nozzles = &settings.project.print.nozzle_diameter.0;
    let total = extruders
        .iter()
        .map(|&index| {
            let diameter = nozzles.get(index).unwrap_or(&nozzles[0]).0;
            crate::geometry::CoordinateScale::Normal
                .checked_scale(diameter)
                .unwrap() as f32
        })
        .sum::<f32>();
    total / extruders.len() as f32
}
