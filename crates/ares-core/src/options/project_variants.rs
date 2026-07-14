mod filament;
mod index;
mod printer;
mod select;

use crate::{
    SliceError,
    options::{OrcaInts, ProjectSettings},
};

pub(crate) fn materialize_project_variants(
    source: &ProjectSettings,
    filament_map: &OrcaInts,
) -> Result<ProjectSettings, SliceError> {
    let mut materialized = source.clone();
    materialized.project.gcode.filament_map = filament_map.clone();

    let Some(active) = index::resolve_activation(&materialized)? else {
        return Ok(materialized);
    };

    let printer_indices = index::resolve_printer_indices(&materialized, &active)?;
    printer::materialize_variant_one(&mut materialized, &printer_indices)?;

    let printer_indices = index::resolve_printer_indices(&materialized, &active)?;
    printer::materialize_variant_two(&mut materialized, &printer_indices)?;

    let process_indices = index::resolve_process_indices(&materialized, &active)?;
    printer::materialize_process(&mut materialized, &process_indices)?;

    let filament_indices = index::resolve_filament_indices(source, filament_map, &active)?;
    filament::materialize(&mut materialized, &filament_indices)?;
    Ok(materialized)
}

#[cfg(test)]
pub(crate) use index::inspect_printer_indices_for_test;
