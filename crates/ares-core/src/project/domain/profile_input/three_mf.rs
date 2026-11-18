use super::{
    OverlayContext, Project, SelectedProfile, SliceError, machine_switch, mapped_prime_tower,
    merged_profile_raw, shift_wipe_tower, shrink_to_new_bed_offset,
};

impl Project {
    /// 3MF selected-profile entry (`OrcaSlicer.cpp:1624-1640` loads the
    /// embedded config as the full-config base; `:2750-2812 update_full_config`
    /// overlays every key the loaded presets supply and keeps the rest).
    ///
    /// `settings` is the selected profile's fully composed config; `profile_raw`
    /// is the chain-expanded presence view and `filament_raws` carries one
    /// merged raw per selected filament for the `:3301` per-slot `set_at`
    /// overlay. Upstream indexes `different_settings_to_system` and
    /// `inherits_group` by `filament_count` (`:2817-2818`). A zero embedded
    /// count takes the loaded count (`:2528-2529`); a nonzero embedded count
    /// still bounds the number of `--load-settings` filaments. A plater
    /// selection's count is its own.
    pub(crate) fn from_3mf_with_profiles(
        input: &[u8],
        profile: SelectedProfile<'_>,
        overrides: &crate::ConfigOverrides,
        (plate, edits): (Option<u32>, &crate::plater::PlaterEdits),
        upward_report: Option<&mut Vec<String>>,
    ) -> Result<(Self, crate::options::flush_matrix::PreparationContext), SliceError> {
        let SelectedProfile {
            settings,
            profile_raw,
            load_names,
            retained_refresh,
            ..
        } = profile;
        let (mut project, preparation) = crate::project::load_project_for_profiles(input)?;
        project.apply_plater_edits(edits);
        let preparation = preparation.for_overrides(overrides);
        let embedded_raw = project.documents.project_settings_raw.clone();
        // The CLI composite entry checks the embedded
        // post_process before the profile overlay (`OrcaSlicer.cpp:1682-1690`).
        if profile.validate_print {
            crate::project_slice::validate_post_process(&project)?;
        }

        let retained_names = load_names.filter(|names| names.embedded.is_some());
        if let Some(names) = retained_names {
            crate::project_slice::compatibility::reject_incompatible_load_settings(
                &embedded_raw,
                names,
                profile_raw,
            )?;
        }
        // OrcaSlicer.cpp:3005 precedes retained preset refresh at :3164–3200.
        if retained_names.is_some()
            && let Some(report) = upward_report
        {
            *report = embedded_raw
                .get("upward_compatible_machine")
                .map(|values| {
                    (0..values.len())
                        .map(|index| values.element(index).unwrap().token().unwrap())
                        .collect()
                })
                .unwrap_or_default();
        }

        let selected = settings.project.preset.filament_settings_id.0.len();
        let embedded_count = project
            .settings()
            .project
            .preset
            .filament_settings_id
            .0
            .len();
        // OrcaSlicer.cpp:2528-2529 uses the loaded count only for zero. The plater's selection (no
        // `--load-settings` names) saves full_fff_config, one filament per selected preset
        // (PresetBundle.cpp:5651-5660, Plater.cpp:19779), so its count is the selection's.
        let filament_count = if embedded_count == 0 || load_names.is_none() {
            selected
        } else {
            embedded_count
        };
        if filament_count == 0 {
            return Err(SliceError::InvalidInput(
                "project embeds no filament selection".to_owned(),
            ));
        }
        if selected > filament_count {
            return Err(SliceError::InvalidInput(format!(
                "selection has {selected} filaments but the project embeds {embedded_count}"
            )));
        }

        let switch = machine_switch(&embedded_raw, settings, project.documents.bbl_family_3mf);
        let mut merge_base = match retained_refresh {
            Some(refresh) => refresh.retained(&embedded_raw, settings)?,
            None => embedded_raw.clone(),
        };
        crate::profiles::apply_retained_preset_defaults(&mut merge_base, Some(settings))?;
        let mut merged = merged_profile_raw(
            &embedded_raw,
            merge_base,
            &profile,
            OverlayContext {
                selected,
                embedded_count: filament_count,
                machine_switch: switch,
            },
        )?;
        // `CLI::run:1820` only records the imported printer name for BBL-family 3MF.
        let original_printer = if project.documents.bbl_family_3mf
            && let Some(super::RawConfigValue::Text(name)) = embedded_raw.get("printer_settings_id")
        {
            name.as_str()
        } else {
            ""
        };
        super::cli_machine_limits::apply(&mut merged, load_names, original_printer);
        // User overrides apply last (`Config.cpp:461`).
        overrides.apply(&mut merged)?;
        mapped_prime_tower::disable_after_mapping(&mut merged, settings, overrides);
        // `set_deserialize`/`handle_legacy` runs on every option token the
        // merged config carries — embedded base and profile overlay alike
        // (`Config.cpp:577-579`) — so the raw view is post-legacy before the
        // typed builder and the `is_value_valid` loop see it.
        merged.handle_legacy();
        // The typed build mirrors upstream's fully parsed `m_print_config`:
        // the bed classification reads it exactly like `opt_float(
        // "printable_height")`/`opt<ConfigOptionPoints>(
        // "printable_area")` over the fully parsed config
        // (`OrcaSlicer.cpp:3711,3708`), `PrintConfigDef` defaults included.
        let mut resolved = crate::options::json_deserialize::deserialize_config_values(
            merged.clone().into_object(),
        )
        .map_err(|error| {
            SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
        })?
        .resolve();
        // OrcaSlicer.cpp:1640 chooses importer membership before placement.
        project.prepare_import_scope(plate);
        // `shrink_to_new_bed` (`OrcaSlicer.cpp:3766-3796`): a BBL-family 3mf
        // re-centers onto the loaded machine's bed — the embedded bed being
        // smaller sets `shrink = 1`, and every instance plus the wipe tower
        // shifts by the bed-center delta (`:3958-4053 translate_models`).
        let offset = project
            .documents
            .bbl_family_3mf
            .then(|| {
                shrink_to_new_bed_offset(&embedded_raw, &resolved, project.instances_world_bbox())
            })
            .flatten();
        if let Some(offset) = offset {
            project.translate_all_instances(offset);
            shift_wipe_tower(&mut merged, offset);
            // The shifted wipe tower reaches the typed view too.
            resolved = crate::options::json_deserialize::deserialize_config_values(
                merged.clone().into_object(),
            )
            .map_err(|error| {
                SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
            })?
            .resolve();
        }

        project.settings = resolved;
        project.documents.project_settings_raw = merged;
        project.initialize_cli_loaded_project_config(filament_count);
        Ok((project, preparation))
    }
}
