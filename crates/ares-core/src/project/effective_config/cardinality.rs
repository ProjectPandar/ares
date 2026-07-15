use crate::{Percent, ProjectSettings, SliceError};

use super::{ValidatedMaterializedProject, invalid_option};

pub(super) fn validate(
    settings: &ProjectSettings,
) -> Result<ValidatedMaterializedProject, SliceError> {
    let physical_extruder_count = settings.project.print.nozzle_diameter.0.len();
    if physical_extruder_count == 0 {
        return Err(invalid_option("nozzle_diameter"));
    }

    let logical_filament_count = settings.filament.gcode.filament_diameter.0.len();
    if logical_filament_count == 0 {
        return Err(invalid_option("filament_diameter"));
    }

    let filament_map = &settings.project.gcode.filament_map.0;
    if filament_map.len() != logical_filament_count
        || filament_map
            .iter()
            .any(|entry| entry.0 <= 0 || entry.0 as usize > physical_extruder_count)
    {
        return Err(invalid_option("filament_map"));
    }

    let filament = &settings.filament;
    validate_minimum_len(
        "filament_ironing_flow",
        filament.region.filament_ironing_flow.len(),
        logical_filament_count,
    )?;
    validate_minimum_len(
        "filament_ironing_spacing",
        filament.region.filament_ironing_spacing.len(),
        logical_filament_count,
    )?;
    validate_minimum_len(
        "filament_ironing_inset",
        filament.region.filament_ironing_inset.len(),
        logical_filament_count,
    )?;
    validate_minimum_len(
        "filament_ironing_speed",
        filament.region.filament_ironing_speed.len(),
        logical_filament_count,
    )?;
    validate_shrink(
        "filament_shrink",
        &filament.print.filament_shrink.0,
        logical_filament_count,
    )?;
    validate_shrink(
        "filament_shrinkage_compensation_z",
        &filament.print.filament_shrinkage_compensation_z.0,
        logical_filament_count,
    )?;

    Ok(ValidatedMaterializedProject {
        physical_extruder_count,
        logical_filament_count,
    })
}

fn validate_minimum_len(key: &str, actual: usize, required: usize) -> Result<(), SliceError> {
    if actual < required {
        return Err(invalid_option(key));
    }
    Ok(())
}

fn validate_shrink(key: &str, values: &[Percent], logical_count: usize) -> Result<(), SliceError> {
    validate_minimum_len(key, values.len(), logical_count)?;
    if values
        .iter()
        .take(logical_count)
        .any(|value| *value != Percent(100.0))
    {
        return Err(SliceError::UnsupportedProjectFeature(key.to_owned()));
    }
    Ok(())
}
