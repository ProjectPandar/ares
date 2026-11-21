use super::{ProjectSettingsRaw, RawConfigValue, SelectedProfile, SliceError};

pub(super) fn compose(
    profile: SelectedProfile<'_>,
    overrides: &crate::ConfigOverrides,
) -> Result<
    (
        crate::ProjectSettings,
        ProjectSettingsRaw,
        crate::options::flush_matrix::PreparationContext,
    ),
    SliceError,
> {
    let mut preparation = crate::options::flush_matrix::PreparationContext::capture(
        &ProjectSettingsRaw::default(),
        overrides,
    )?;
    // STL --load-settings is not the BBL 3MF filament-mapping branch
    // (OrcaSlicer.cpp:2275–2277); apply user edits to the composed profile.
    // CLI preset identities are injected after loading settings
    // (OrcaSlicer.cpp:2048,2088,3152–3158,3175–3180), before
    // command-line overrides (:3613–3615). Use the same composed data
    // for both STL paths (profiles/composition.rs:186–211).
    let project = &profile.settings.project;
    let sparse = profile.load_names.is_some()
        && project
            .preset
            .filament_settings_id
            .0
            .iter()
            .any(String::is_empty);
    // Sparse composition already expands vectors to every CLI slot; the
    // ordinary overlay only sees loaded files and would compact the gaps.
    let ordinary = if sparse {
        ProjectSettingsRaw::default()
    } else {
        crate::profiles::loaded_filament_ordinary_values(profile.filament_raws)?
    };
    let mut raw = profile.profile_raw.clone();
    raw.extend_raw(&ordinary);
    let capped = super::cli_machine_limits::apply(&mut raw, profile.load_names, "");
    preparation.sparse_filament_slots = sparse;
    for (key, value) in [
        ("print_settings_id", &project.preset.print_settings_id.0),
        ("printer_settings_id", &project.preset.printer_settings_id.0),
    ] {
        raw.insert(key.to_owned(), RawConfigValue::Text(value.clone()));
    }
    for (key, values) in [
        (
            "filament_settings_id",
            &project.preset.filament_settings_id.0,
        ),
        ("filament_ids", &project.gcode.filament_ids.0),
        (
            "print_compatible_printers",
            &project.preset.print_compatible_printers.0,
        ),
    ] {
        raw.insert_strings(key, values);
    }
    if let Some(groups) = profile.full_fff_config {
        groups.apply(&mut raw);
    }
    for (key, values) in [
        ("filament_map", &project.gcode.filament_map.0),
        ("filament_self_index", &project.preset.filament_self_index.0),
    ] {
        raw.insert(
            key.to_owned(),
            RawConfigValue::List(
                values
                    .iter()
                    .map(|value| RawConfigValue::Text(value.0.to_string()))
                    .collect(),
            ),
        );
    }
    let settings = if overrides.is_empty() {
        // The composed profile raw is already the post-`handle_legacy`
        // view (`Config.cpp:577-579` normalizes every loaded preset at
        // compose, before any overlay).
        let mut settings = profile.settings.clone();
        if !sparse {
            let (_, _, ordinary, _) =
                crate::options::json_deserialize::deserialize_config_values(ordinary.into_object())
                    .map_err(SliceError::InvalidInput)?
                    .into_profile_parts();
            ordinary.apply_present(&mut settings.filament);
        }
        if capped {
            settings.printer.machine = crate::options::json_deserialize::deserialize_config_values(
                raw.clone().into_object(),
            )
            .map_err(|error| {
                SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
            })?
            .resolve()
            .printer
            .machine;
        }
        // CLI:4250-4259 adds a missing static default once, not once per
        // loaded filament. Typed composition must not invent raw members.
        settings
            .filament
            .restore_absent_defaults(|key| raw.contains_key(key));
        settings
    } else {
        overrides.apply(&mut raw)?;
        // `handle_legacy` on user edits (Config.cpp:577–579) precedes
        // the typed settings rebuild.
        raw.handle_legacy();
        crate::options::json_deserialize::deserialize_config_values(raw.clone().into_object())
            .map_err(|error| {
                SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
            })?
            .resolve()
    };
    // CLI:4243-4244 normalizes mixed metadata for the loaded logical slots.
    let filament_count = settings.project.preset.filament_settings_id.0.len();
    super::resize_mixed_filament_metadata(&mut raw, filament_count, filament_count);
    Ok((settings, raw, preparation))
}
