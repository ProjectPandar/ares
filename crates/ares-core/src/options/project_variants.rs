mod filament;
mod filament_maps;
mod filament_retract;
mod filament_select;

pub(crate) use filament_maps::update_static_filament_maps;
mod index;
mod printer;
mod process;
mod select;

pub(crate) use index::{canonical_variant, get_index_for_extruder};

use crate::{
    Nullable, SliceError,
    options::{OrcaInt, OrcaInts, ProjectSettings, project_config_views::ProjectConfigViews},
};

pub(crate) struct MaterializedProjectVariants {
    pub(crate) full: ProjectSettings,
    pub(crate) checkpoint: ProjectSettings,
    // PrintApply.cpp:1413-1432 retains this selection for region overlays.
    pub(crate) process_variant_indices: Vec<usize>,
}

pub(crate) fn materialize_project_variants(
    source: &ProjectSettings,
    filament_map: &OrcaInts,
) -> Result<MaterializedProjectVariants, SliceError> {
    let mut materialized = source.clone();
    let filament_map = expanded_filament_map(source, filament_map);
    materialized.project.gcode.filament_map = filament_map.clone();

    let Some(active) = index::resolve_activation(&materialized)? else {
        let checkpoint = materialized.clone();
        return Ok(MaterializedProjectVariants {
            full: materialized,
            checkpoint,
            process_variant_indices: vec![0],
        });
    };

    let process_indices = if active.extruders {
        // PrintApply.cpp:1423-1427: variant one shrinks the id/variant metadata,
        // so resolve and materialize the stride-two values first.
        let printer_indices =
            index::resolve_printer_indices_for_variant_two(&materialized, &active.variants);
        printer::materialize_variant_two(&mut materialized, &printer_indices)?;

        let printer_indices = index::resolve_printer_indices(&materialized, &active.variants)?;
        printer::materialize_variant_one(&mut materialized, &printer_indices)?;

        index::ensure_process_variant_columns(&mut materialized);
        let process_indices = index::resolve_process_indices(&materialized, &active.variants)?;
        process::materialize(&mut materialized, &process_indices)?;
        process_indices
    } else {
        // PrintApply.cpp:1431-1432
        vec![0]
    };

    // PrintApply.cpp:1165-1172 preserves this post-printer/process snapshot
    // before the multiple-filament variant materialization below.
    let checkpoint = materialized.clone();
    let filament_indices =
        index::resolve_filament_indices(source, &filament_map, &active.variants)?;
    filament::materialize(&mut materialized, &checkpoint, &filament_indices);
    remap_filament_self_index(&mut materialized, &checkpoint, &filament_indices);
    Ok(MaterializedProjectVariants {
        full: materialized,
        checkpoint,
        process_variant_indices: process_indices,
    })
}

pub(crate) fn refresh_filament_map(
    checkpoint: &ProjectSettings,
    filament_map: &OrcaInts,
    current: &ProjectSettings,
) -> Result<ProjectConfigViews, SliceError> {
    // Print.cpp:3171-3175 assigns the recommendation without CLI-entry growth.
    let filament_map = filament_map.clone();
    let mut refreshed = current.clone();
    refreshed.project.gcode.filament_map = filament_map.clone();
    if let Some(active) = index::resolve_activation(checkpoint)? {
        let filament_indices =
            index::resolve_filament_indices(checkpoint, &filament_map, &active.variants)?;
        // Print.cpp:3174-3195: select from the saved full config, then apply
        // only filament_options_with_variant to m_config, not process options.
        filament::materialize(&mut refreshed, checkpoint, &filament_indices);
        remap_filament_self_index(&mut refreshed, checkpoint, &filament_indices);
    }
    // Print.cpp:3193-3195 applies the newly computed retract overrides last.
    crate::options::project_config_views::resolve_project_config_views(refreshed)
}

// Print.cpp:5008-5015,5153-5155: without a layered grouping producer,
// the nozzle config index is the mapped physical extruder, not a variant search.
// Layered/dynamic multi-nozzle grouping remains outside this rewrite slice.
pub(crate) fn mapped_extruder_id(settings: &ProjectSettings, filament: usize) -> usize {
    settings
        .project
        .gcode
        .filament_map
        .0
        .get(filament)
        .map_or(0, |id| (id.0 - 1) as usize)
}

// PrintObject.cpp:4363-4374; PrintConfig.cpp:11421-11429; Config.hpp:612-629.
// Non-nil singletons broadcast. Nil/out-of-range source columns retain the target.
pub(in crate::options) fn apply_nullable_vector<T: Clone>(
    target: &mut Vec<Nullable<T>>,
    source: &[Nullable<T>],
    indices: &[usize],
) {
    if indices.is_empty() {
        target.clear();
        target.extend_from_slice(source);
        return;
    }
    target.resize(indices.len(), source[0].clone());
    let broadcast = source.len() == 1 && !matches!(source[0], Nullable::Nil);
    for (value, &index) in target.iter_mut().zip(indices) {
        let index = if broadcast { 0 } else { index };
        if let Some(selected @ Nullable::Value(_)) = source.get(index) {
            value.clone_from(selected);
        }
    }
}

fn expanded_filament_map(source: &ProjectSettings, map: &OrcaInts) -> OrcaInts {
    let mut expanded = map.clone();
    let filament_count = source.filament.gcode.filament_diameter.0.len();
    if expanded.0.len() < filament_count {
        expanded.0.resize(filament_count, OrcaInt(1));
    }
    if source.project.print.nozzle_diameter.0.len() == 1 {
        expanded.0[..filament_count].fill(OrcaInt(1));
    }
    expanded
}

// OrcaSlicer 2.5.0 PrintConfig.cpp:11954-11962 remaps IDs last with get_at;
// Config.hpp:676-680 reuses the first ID for an index beyond the list length.
fn remap_filament_self_index(
    settings: &mut ProjectSettings,
    source: &ProjectSettings,
    indices: &[usize],
) {
    let ids = &source.project.preset.filament_self_index.0;
    let Some(first) = ids.first() else {
        return;
    };
    settings.project.preset.filament_self_index = OrcaInts(
        indices
            .iter()
            .map(|&index| *ids.get(index).unwrap_or(first))
            .collect(),
    );
}
