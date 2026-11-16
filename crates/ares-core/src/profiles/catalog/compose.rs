//! Selection composition: per-kind merge of loaded records onto the
//! raw overlay (`update_full_config` per-key apply, `OrcaSlicer.cpp:2750-2812`).

use std::borrow::Cow;

use super::super::{
    MergedProfile, ProfileFragment, ProfileKind, ProfileSelection,
    composition::compose_resolved_profiles,
    fragment::payload::ProfilePayload,
    inheritance::{UnselectedKind, merge_direct_profile_fragment, unselected_profile},
    merge_profile_fragments,
};
use super::invalid;
use super::{Catalog, ComposedSelection};
use crate::{
    SliceError,
    options::{FilamentOptions, ProjectRuntimeOptionsBuilder},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// `OrcaSlicer.cpp:546` gcodes key set created on the machine config by
/// `load_default_gcodes_to_config` (`:548-586`).
const PRINTER_GCODE_KEYS: [&str; 10] = [
    "change_filament_gcode",
    "layer_change_gcode",
    "machine_end_gcode",
    "machine_pause_gcode",
    "machine_start_gcode",
    "template_custom_gcode",
    "printing_by_object_gcode",
    "before_layer_change_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
];

/// `OrcaSlicer.cpp:587-606` filament keys resized to one empty slot.
const FILAMENT_GCODE_KEYS: [&str; 2] = ["filament_start_gcode", "filament_end_gcode"];

impl Catalog {
    pub(in crate::profiles) fn compose(
        &self,
        selection: &ProfileSelection,
    ) -> Result<ComposedSelection, SliceError> {
        let mut raw = ProjectSettingsRaw::default();
        let mut runtime = ProjectRuntimeOptionsBuilder::default();
        // OrcaSlicer.cpp:2849-2867,2993-3011 applies machine, then process.
        let machine = match selection.machine() {
            Some(name) => {
                let merged = self.merge(ProfileKind::Machine, name, &mut raw, &mut runtime)?;
                // `load_default_gcodes_to_config(load_machine_config,
                // TYPE_PRINTER)` (`OrcaSlicer.cpp:2850,548-586`): every
                // printer G-code option exists on the machine config,
                // created with the option default — empty except the two
                // templates (PrintConfig.cpp:5947,2035) — before
                // `update_full_config` overlays the machine onto the
                // embedded project settings. With no machine loaded
                // there is no machine config to default (`:2009-2106`).
                insert_default_options(&mut raw, &PRINTER_GCODE_KEYS);
                merged
            }
            None => unselected_profile(UnselectedKind::Machine),
        };
        let process = match selection.process() {
            Some(name) => self.merge(ProfileKind::Process, name, &mut raw, &mut runtime)?,
            None => unselected_profile(UnselectedKind::Process),
        };
        // Each filament keeps its own merged raw for the 3MF per-slot overlay
        // (`OrcaSlicer.cpp:3301` set_at per loaded filament index).
        let mut filament_raws = Vec::with_capacity(selection.filaments().len());
        let filaments = selection
            .filaments()
            .iter()
            .map(|name| {
                let mut filament_raw = ProjectSettingsRaw::default();
                let mut merged =
                    self.merge(ProfileKind::Filament, name, &mut filament_raw, &mut runtime)?;
                let MergedProfile::Filament { options, .. } = &mut merged else {
                    unreachable!("filament merge changed profile kind")
                };
                // `load_default_gcodes_to_config(config, TYPE_FILAMENT)`
                // (`OrcaSlicer.cpp:3590,710-728`): an empty filament
                // start/end G-code vector is resized to one empty slot.
                insert_default_options(&mut filament_raw, &FILAMENT_GCODE_KEYS);
                resize_empty_filament_gcodes(&mut filament_raw, options);
                if self.direct_imports {
                    // CLI::run creates this option per loaded file slot before variant matching.
                    // OrcaSlicer.cpp:3625-3635.
                    insert_default_options(&mut filament_raw, &["filament_extruder_variant"]);
                }
                raw.extend_raw(&filament_raw);
                filament_raws.push(filament_raw);
                Ok(merged)
            })
            .collect::<Result<Vec<_>, SliceError>>()?;
        super::cli_metadata::filament_conditions(&mut raw, &filaments);
        // File-slot lookup keys are not exported preset names.
        let source_selection = if self.direct_imports && !selection.filaments().is_empty() {
            Cow::Owned(ProfileSelection::new_partial(
                selection.process().map(str::to_owned),
                selection.machine().map(str::to_owned),
                selection.filaments().iter().map(|key| {
                    let position = self.selectable[&(ProfileKind::Filament, key.clone())];
                    self.document(position).name.clone()
                }),
            )?)
        } else {
            Cow::Borrowed(selection)
        };
        let mut composed = compose_resolved_profiles(
            &source_selection,
            machine,
            process,
            filaments,
            runtime.resolve(),
        );
        // OrcaSlicer.cpp:2250–2255,3616–3622: loaded identity is authored,
        // even when the resolved system preset inherits a nonempty ID.
        composed.settings.project.gcode.filament_ids = crate::OrcaStrings(
            selection
                .filaments()
                .iter()
                .map(|name| {
                    let position = self.selectable[&(ProfileKind::Filament, name.clone())];
                    super::document_text(self.document(position), "filament_id")
                })
                .collect(),
        );
        if let Some(group) = composed.metadata().inherits_group() {
            raw.insert_strings("inherits_group", group);
        }
        // Every loaded preset file normalizes through
        // `set_deserialize`/`handle_legacy` at load (`Config.cpp:577-579`,
        // `PrintConfig.cpp:8033`), before `update_full_config` overlays it
        // onto the embedded project (`OrcaSlicer.cpp:2750-2812`); the
        // embedded base keeps its own post-merge normalization in the
        // profile entries.
        raw.handle_legacy();
        for filament_raw in &mut filament_raws {
            filament_raw.handle_legacy();
        }
        Ok(ComposedSelection {
            profile: composed,
            raw,
            filament_raws,
        })
    }

    fn merge(
        &self,
        kind: ProfileKind,
        name: &str,
        raw: &mut ProjectSettingsRaw,
        runtime: &mut ProjectRuntimeOptionsBuilder,
    ) -> Result<MergedProfile, SliceError> {
        let position = *self
            .selectable
            .get(&(kind, name.to_owned()))
            .ok_or_else(|| invalid(format!("selectable profile '{name}' was not found")))?;
        // Typed sparse files bypass preset resolution (OrcaSlicer.cpp:2121-2123).
        let layers = if self.records[position].direct {
            vec![Cow::Borrowed(self.document(position))]
        } else {
            self.resolved_layers(position)?
        };
        // Preset.cpp:1925–1944 / PresetBundle.cpp:7827–7839 carry collection defaults
        // through resolved user and vendor presets.
        let completed_vendor = matches!(self.records[position].scope, super::Scope::Vendor(_))
            && self.document(position).instantiation.as_deref() == Some("true");
        let mut defaults =
            if completed_vendor || (!self.direct_imports && self.is_user_scope(position)) {
                super::super::project_refresh::default_preset_fields(kind)?
            } else {
                Default::default()
            };
        raw.extend(
            defaults
                .iter()
                .filter(|(key, _)| super::raw::applies(kind, key))
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        let mut fragments = Vec::with_capacity(layers.len());
        for layer in layers {
            let (fragment, project) = ProfileFragment::from_catalog_values(
                layer.values(),
                std::mem::take(&mut defaults),
            )?;
            runtime.overlay(project);
            let mut authored = ProjectSettingsRaw::default();
            authored.extend(
                layer
                    .fields
                    .iter()
                    .filter(|(key, _)| super::raw::applies(kind, key))
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
            // Config.cpp:600-615 normalizes each source before PresetBundle.cpp:7839
            // applies it over the parent/default config, not over a joined raw map.
            authored.handle_legacy();
            if completed_vendor {
                super::super::preset_normalization::replace_nil_from_defaults(&mut authored, raw)?;
            }
            raw.extend_raw(&authored);
            // Config.cpp:1147-1149 converts authored JSON before preset completion.
            if layer.fields.contains_key("thumbnails")
                && let ProfilePayload::Machine(printer) = fragment.payload()
                && let Some(thumbnails) = printer.present_thumbnails()
            {
                raw.insert(
                    "thumbnails".to_owned(),
                    RawConfigValue::Text(thumbnails.to_owned()),
                );
            }
            if completed_vendor {
                super::super::preset_normalization::extend_default_config_length(raw)?;
            }
            fragments.push(fragment);
        }
        let mut merged = if self.records[position].direct {
            merge_direct_profile_fragment(&fragments, kind, &self.document(position).name)?
        } else {
            merge_profile_fragments(&fragments, kind, &self.document(position).name)?
        };
        if kind == ProfileKind::Process && !self.direct_imports && self.is_user_scope(position) {
            // Preset.cpp:1925–1944: user process completion precedes CLI variant selection.
            super::super::preset_normalization::complete_process_variants(raw)?;
            let (_, process, _, _) = crate::options::json_deserialize::deserialize_config_values(
                raw.clone().into_object(),
            )
            .map_err(SliceError::InvalidInput)?
            .into_profile_parts();
            let MergedProfile::Process { options, .. } = &mut merged else {
                unreachable!("process selection has a process payload")
            };
            **options = process.resolve();
        }
        if kind == ProfileKind::Machine && !self.direct_imports && self.is_user_scope(position) {
            super::super::preset_normalization::complete_machine_variants(
                raw,
                self.document(position)
                    .inherits
                    .as_deref()
                    .is_some_and(|parent| !parent.is_empty()),
            )?;
        }
        if kind == ProfileKind::Machine
            && !self.direct_imports
            && (super::super::preset_normalization::normalize_machine(raw)?
                || completed_vendor
                || self.is_user_scope(position))
        {
            let (printer, _, _, project) =
                crate::options::json_deserialize::deserialize_config_values(
                    raw.clone().into_object(),
                )
                .map_err(SliceError::InvalidInput)?
                .into_profile_parts();
            let MergedProfile::Machine { options, .. } = &mut merged else {
                unreachable!("machine selection has a machine payload")
            };
            **options = printer.resolve();
            runtime.overlay(project);
        }
        if !self.is_user_scope(position) {
            let filament_id = fragments
                .iter()
                .rev()
                .filter_map(ProfileFragment::filament_id)
                .find(|value| !value.is_empty())
                .map(str::to_owned);
            merged.apply_system_metadata(filament_id);
        }
        Ok(merged)
    }
}

/// OrcaSlicer.cpp:710-728, `load_default_gcodes_to_config(TYPE_FILAMENT)`:
/// resize empty start/end G-code vectors in both raw and merged filament options.
fn resize_empty_filament_gcodes(raw: &mut ProjectSettingsRaw, filament: &mut FilamentOptions) {
    for (key, gcodes) in [
        (
            "filament_start_gcode",
            &mut filament.gcode.filament_start_gcode.0,
        ),
        (
            "filament_end_gcode",
            &mut filament.gcode.filament_end_gcode.0,
        ),
    ] {
        let empty = match raw.get(key).unwrap() {
            RawConfigValue::Text(token) => matches!(
                crate::options::json_deserialize::decode(crate::OptionValueKind::Strings, token)
                    .unwrap(),
                RawConfigValue::List(values) if values.is_empty()
            ),
            RawConfigValue::List(values) => values.is_empty(),
            _ => unreachable!("loaded filament G-code is a string vector"),
        };
        if !empty {
            continue;
        }
        gcodes.resize(1, String::new());
        raw.insert_strings(key, gcodes);
    }
}

/// `option(key, true)` with the option default (`Config.hpp` create path):
/// absent keys are created from the registry default, present keys stay.
/// Every inserted value is the registered `OptionDefinition::default_value`.
fn insert_default_options(raw: &mut ProjectSettingsRaw, keys: &[&str]) {
    for definition in crate::option_definitions() {
        if !keys.contains(&definition.key) {
            continue;
        }
        if !raw.contains_key(definition.key) {
            raw.insert(
                definition.key.to_owned(),
                RawConfigValue::Text(definition.default_value.to_owned()),
            );
        }
    }
}
