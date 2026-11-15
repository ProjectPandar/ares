use serde::Deserialize;
use serde_json::Value;

use crate::{
    SliceError, SliceOptions,
    options::{OrcaBool, json_deserialize, registry},
    project::raw_settings::ProjectSettingsRaw,
};

// Preset.cpp:338-350: replace a whole nil vector against the unchanged parent/default
// before overlay; extend_default_config_length then resizes the completed layer.
pub(super) fn replace_nil_from_defaults(
    authored: &mut ProjectSettingsRaw,
    defaults: &ProjectSettingsRaw,
) -> Result<(), SliceError> {
    let options =
        json_deserialize::deserialize_preset_config::<SliceOptions>(authored.clone().into_object())
            .map_err(SliceError::InvalidInput)?;
    let mut replacements = Vec::new();
    for (key, _) in authored.entries() {
        if !registry::option_metadata(key).is_some_and(|metadata| metadata.nullable)
            || super::project_refresh::keys::is_filament_extruder_override_key(key)
            || !(registry::print_options_with_variant().contains(&key.as_str())
                || registry::filament_options_with_variant().contains(&key.as_str())
                || registry::printer_options_with_variant_1().contains(&key.as_str())
                || registry::printer_options_with_variant_2().contains(&key.as_str()))
        {
            continue;
        }
        let Some(default) = defaults.get(key) else {
            continue;
        };
        let Value::Array(values) = &options.values()[key] else {
            unreachable!("nullable preset variant options are vectors")
        };
        if values.iter().all(|value| value == "nil") {
            replacements.push((key.clone(), default.clone()));
        }
    }
    authored.extend(replacements);
    Ok(())
}

pub(super) fn extend_default_config_length(raw: &mut ProjectSettingsRaw) -> Result<(), SliceError> {
    let options =
        json_deserialize::deserialize_preset_config::<SliceOptions>(raw.clone().into_object())
            .map_err(SliceError::InvalidInput)?;
    for (key, value) in options.extended_preset_variants()? {
        raw.insert(key, serde_json::from_value(value).unwrap());
    }
    Ok(())
}

pub(super) fn complete_process_variants(raw: &mut ProjectSettingsRaw) -> Result<(), SliceError> {
    let mut process = ProjectSettingsRaw::default();
    process.extend(
        raw.entries()
            .filter(|(key, _)| registry::print_options_with_variant().contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    extend_default_config_length(&mut process)?;
    raw.extend_raw(&process);
    Ok(())
}

// Preset.cpp:535-541 runs once on the completed machine preset, not each parent.
pub(super) fn normalize_machine(raw: &mut ProjectSettingsRaw) -> Result<bool, SliceError> {
    let mut input = ProjectSettingsRaw::default();
    input.extend(
        raw.entries()
            .filter(|(key, _)| {
                matches!(
                    key.as_str(),
                    "single_extruder_multi_material"
                        | "extruder_variant_list"
                        | "printer_extruder_id"
                        | "printer_extruder_variant"
                        | "print_extruder_variant"
                        | "filament_extruder_variant"
                ) || registry::extruder_option_keys().contains(&key.as_str())
                    || registry::printer_options_with_variant_2().contains(&key.as_str())
            })
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    let mut options =
        json_deserialize::deserialize_preset_config::<SliceOptions>(input.into_object())
            .map_err(SliceError::InvalidInput)?;
    let Some(single_material) = options.values().get("single_extruder_multi_material") else {
        return Ok(false);
    };
    if OrcaBool::deserialize(single_material.clone())
        .map_err(|error| SliceError::InvalidInput(error.to_string()))?
        .0
    {
        return Ok(false);
    }
    let Some(Value::Array(nozzles)) = options.values().get("nozzle_diameter") else {
        return Ok(false);
    };
    options.set_num_extruders(nozzles.len())?;
    for key in [
        "extruder_variant_list",
        "printer_extruder_id",
        "printer_extruder_variant",
    ]
    .into_iter()
    .chain(registry::extruder_option_keys().iter().copied())
    .chain(registry::printer_options_with_variant_2().iter().copied())
    .filter(|key| *key != "default_filament_profile")
    {
        if let Some(value) = options.values().get(key) {
            raw.insert(
                key.to_owned(),
                serde_json::from_value(value.clone()).unwrap(),
            );
        }
    }
    Ok(true)
}
