use super::ProjectSettingsRaw;
use crate::{
    ConfigOverrides, ProjectSettings, SliceError,
    options::{json_deserialize, registry::OptionValueKind},
    project::raw_settings::RawConfigValue,
};

pub(super) fn saved_filament_limit(raw: &ProjectSettingsRaw) -> i32 {
    let count = match raw.get("filament_settings_id") {
        Some(RawConfigValue::List(ids)) => Some(ids.len()),
        Some(RawConfigValue::Text(wire)) => {
            match json_deserialize::decode(OptionValueKind::Strings, wire) {
                Ok(RawConfigValue::List(ids)) => Some(ids.len()),
                _ => None,
            }
        }
        _ => None,
    };
    count
        .and_then(|count| i32::try_from(count).ok())
        .unwrap_or(i32::MAX)
}

pub(super) fn resolve(
    mut project_settings_raw: ProjectSettingsRaw,
    overrides: &ConfigOverrides,
    materialize_retained_defaults: bool,
) -> Result<(ProjectSettingsRaw, ProjectSettings), SliceError> {
    // Preset.cpp:1059-1089 constructs retained presets over their full defaults,
    // excluding embedded connection credentials from the project overlay.
    if materialize_retained_defaults {
        crate::profiles::apply_retained_preset_defaults(&mut project_settings_raw, None)?;
    }
    // Overrides win over document values (`apply_only`, `Config.cpp:461`).
    overrides.apply(&mut project_settings_raw)?;
    // Same upstream deserializer semantics as preset documents: JSON
    // strings on vector-kind options deserialize through
    // `ConfigOptionVector::deserialize` (`Config.hpp:868-887`,
    // `ConfigOptionStrings::deserialize`, `Config.hpp:1153-1159`) — a bare
    // scalar is a single-element vector, an empty string an empty one.
    project_settings_raw
        .normalize_wire_vector_shapes()
        .map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    // Caller overrides are a later serialized write (`Config.cpp:602`);
    // normalize their tokens without repeating an embedded-only load pass.
    if !overrides.is_empty() {
        project_settings_raw.handle_legacy();
    }
    // The array branch of the same upstream load: joined string arrays bind
    // scalar options (`Config.cpp:945-1001`), so the validation flat map and
    // the typed builders see the bound scalar, not the array.
    project_settings_raw
        .normalize_scalar_arrays()
        .map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    let mut materialized_settings_raw = project_settings_raw.clone();
    materialized_settings_raw.complete_filament_variant_defaults()?;
    let settings = <crate::options::ProjectSettingsBuilder as serde::Deserialize>::deserialize(
        materialized_settings_raw.into_object(),
    )
    .map_err(|error| SliceError::InvalidInput(format!("invalid project settings JSON: {error}")))?
    .resolve();

    Ok((project_settings_raw, settings))
}
