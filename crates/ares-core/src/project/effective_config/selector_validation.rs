use crate::{OrcaInt, ProjectSettings, SliceError};

use super::{Project, ValidatedMaterializedProject, invalid_option};

pub(super) fn validate(
    settings: &ProjectSettings,
    project: &Project,
    validated: ValidatedMaterializedProject,
) -> Result<(), SliceError> {
    validate_wipe_tower(settings, validated)?;
    validate_support_selector("support_filament", settings.process.object.support_filament)?;
    validate_support_selector(
        "support_interface_filament",
        settings.process.object.support_interface_filament,
    )?;

    for object in project.objects() {
        if let Some(selector) = object.object_overrides().support_filament {
            validate_support_selector("support_filament", selector)?;
        }
        if let Some(selector) = object.object_overrides().support_interface_filament {
            validate_support_selector("support_interface_filament", selector)?;
        }

        validate_raw_extruder(
            object.region_overrides().extruder,
            validated.logical_filament_count,
        )?;
        for volume in object.volumes() {
            validate_raw_extruder(
                volume.region_overrides().extruder,
                validated.logical_filament_count,
            )?;
        }
        for layer_range in object.layer_config_ranges() {
            validate_raw_extruder(
                layer_range.region_overrides().extruder,
                validated.logical_filament_count,
            )?;
        }
    }
    Ok(())
}

fn validate_wipe_tower(
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
) -> Result<(), SliceError> {
    let selector = settings.process.print.wipe_tower_filament.0;
    if selector == 0 {
        return Ok(());
    }
    if selector < 0 {
        return Err(invalid_option("wipe_tower_filament"));
    }

    let selector = selector as usize;
    if selector >= validated.physical_extruder_count || selector > validated.logical_filament_count
    {
        return Err(invalid_option("wipe_tower_filament"));
    }
    Ok(())
}

fn validate_support_selector(key: &str, selector: OrcaInt) -> Result<(), SliceError> {
    if selector.0 < 0 {
        return Err(invalid_option(key));
    }
    Ok(())
}

fn validate_raw_extruder(
    extruder: Option<OrcaInt>,
    logical_filament_count: usize,
) -> Result<(), SliceError> {
    if let Some(extruder) = extruder
        && (extruder.0 < 0 || extruder.0 as usize > logical_filament_count)
    {
        return Err(invalid_option("extruder"));
    }
    Ok(())
}
