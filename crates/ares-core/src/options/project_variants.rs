mod filament;
mod filament_select;
mod index;
mod printer;
mod select;

pub(crate) use index::get_index_for_extruder;

use crate::{
    SliceError,
    options::{OrcaInt, OrcaInts, ProjectSettings, project_config_views::ProjectConfigViews},
};

pub(crate) struct MaterializedProjectVariants {
    pub(crate) full: ProjectSettings,
    pub(crate) checkpoint: ProjectSettings,
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
        });
    };

    let printer_indices = index::resolve_printer_indices(&materialized, &active)?;
    printer::materialize_variant_one(&mut materialized, &printer_indices)?;

    let printer_indices = index::resolve_printer_indices_for_variant_two(&materialized, &active);
    printer::materialize_variant_two(&mut materialized, &printer_indices)?;

    let process_indices = index::resolve_process_indices(&materialized, &active)?;
    printer::materialize_process(&mut materialized, &process_indices)?;

    // PrintApply.cpp:1165-1172 preserves this post-printer/process snapshot
    // before the multiple-filament variant materialization below.
    let checkpoint = materialized.clone();
    let filament_indices = index::resolve_filament_indices(source, &filament_map, &active)?;
    filament::materialize(&mut materialized, &checkpoint, &filament_indices)?;
    Ok(MaterializedProjectVariants {
        full: materialized,
        checkpoint,
    })
}

#[cfg(test)]
pub(crate) fn materialize_project_variants_full(
    source: &ProjectSettings,
    filament_map: &OrcaInts,
) -> Result<ProjectSettings, SliceError> {
    materialize_project_variants(source, filament_map).map(|value| value.full)
}

pub(crate) fn refresh_filament_map(
    checkpoint: &ProjectSettings,
    filament_map: &OrcaInts,
    current: &ProjectSettings,
) -> Result<ProjectConfigViews, SliceError> {
    let filament_map = expanded_filament_map(checkpoint, filament_map);
    let mut refreshed = current.clone();
    refreshed.project.gcode.filament_map = filament_map.clone();
    if let Some(active) = index::resolve_activation(checkpoint)? {
        let filament_indices = index::resolve_filament_indices(checkpoint, &filament_map, &active)?;
        // Print.cpp:3174-3195: select from the saved full config, then apply
        // only filament_options_with_variant to m_config, not process options.
        filament::materialize(&mut refreshed, checkpoint, &filament_indices)?;
    }
    // Print.cpp:3193-3195 applies the newly computed retract overrides last.
    crate::options::project_config_views::resolve_project_config_views(refreshed)
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

#[cfg(test)]
pub(crate) use index::inspect_printer_indices_for_test;
