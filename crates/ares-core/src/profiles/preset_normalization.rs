use serde::Deserialize;
use serde_json::Value;

use crate::{
    ProfileKind, SliceError, SliceOptions,
    options::{OrcaBool, json_deserialize, registry},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
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

// Preset.cpp:375-389 pads inherited vectors; roots use the full extension at1944.
pub(super) fn complete_machine_variants(
    raw: &mut ProjectSettingsRaw,
    inherited: bool,
) -> Result<(), SliceError> {
    let mut printer = ProjectSettingsRaw::default();
    printer.extend(
        raw.entries()
            .filter(|(key, _)| {
                key.as_str() == "nozzle_diameter"
                    || registry::printer_options_with_variant_1().contains(&key.as_str())
                    || registry::printer_options_with_variant_2().contains(&key.as_str())
            })
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    let options =
        json_deserialize::deserialize_preset_config::<SliceOptions>(printer.into_object())
            .map_err(SliceError::InvalidInput)?;
    if inherited
        && options
            .values()
            .get("printer_extruder_variant")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Ok(());
    }
    for (key, value) in options.extended_preset_variants()? {
        let source = options.values()[&key].as_array().unwrap();
        let target = value.as_array().unwrap();
        if !inherited || (!source.is_empty() && target.len() > source.len()) {
            raw.insert(key, serde_json::from_value(value).unwrap());
        }
    }
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

// Preset::normalize (Preset.cpp:555-599) on each preset PresetCollection::load_presets loads (:1947).
pub(super) fn normalize(raw: &mut ProjectSettingsRaw, kind: ProfileKind) -> Result<(), SliceError> {
    match kind {
        ProfileKind::Machine => normalize_machine(raw).map(drop),
        ProfileKind::Filament => normalize_filament(raw),
        ProfileKind::Process => Ok(()),
    }
}

// Preset.cpp:576-598: a loaded filament preset keeps one value per filament (its
// `filament_diameter` count) in every filament vector but the variant, dev and compatibility ones.
fn normalize_filament(raw: &mut ProjectSettingsRaw) -> Result<(), SliceError> {
    let Some(size) = raw.get("filament_diameter").map(RawConfigValue::len) else {
        return Ok(());
    };
    for key in super::project_refresh::keys::names(ProfileKind::Filament) {
        if matches!(key, "compatible_prints" | "compatible_printers")
            || registry::filament_options_with_variant().contains(&key)
            || super::project_refresh::dev_key(key)
            || !crate::option_definition(key)
                .is_some_and(|definition| super::project_refresh::is_vector(definition.kind))
        {
            continue;
        }
        let Some(RawConfigValue::List(values)) = raw.get(key) else {
            continue;
        };
        if values.len() == size {
            continue;
        }
        let mut values = values.clone();
        // Config.hpp:687-716: extend with the vector's own first value, or the default's.
        let fill = match values.first() {
            Some(first) => first.clone(),
            None => match super::project_refresh::keys::default_value(key)? {
                Some(RawConfigValue::List(defaults)) => defaults[0].clone(),
                default => unreachable!("{key} has a vector default: {default:?}"),
            },
        };
        values.resize(size, fill);
        raw.insert(key.to_owned(), RawConfigValue::List(values));
    }
    if let Some(RawConfigValue::List(names)) = raw.get("filament_settings_id") {
        let mut names = names.clone();
        names.resize(size, RawConfigValue::Text(String::new()));
        raw.insert(
            "filament_settings_id".to_owned(),
            RawConfigValue::List(names),
        );
    }
    Ok(())
}
