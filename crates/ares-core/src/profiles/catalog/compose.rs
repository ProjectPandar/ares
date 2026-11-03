//! Selection composition: per-kind merge of loaded records onto the
//! raw overlay (`update_full_config` per-key apply, `OrcaSlicer.cpp:2750-2812`).

use std::borrow::Cow;

use super::super::{
    MergedProfile, ProfileFragment, ProfileKind, ProfileSelection,
    composition::compose_resolved_profiles,
    inheritance::{UnselectedKind, merge_direct_profile_fragment, unselected_profile},
    merge_profile_fragments,
};
use super::invalid;
use super::{Catalog, ComposedSelection};
use crate::{
    SliceError,
    options::ProjectRuntimeOptionsBuilder,
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
                insert_default_gcodes(&mut raw, &PRINTER_GCODE_KEYS);
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
                let merged =
                    self.merge(ProfileKind::Filament, name, &mut filament_raw, &mut runtime)?;
                // `load_default_gcodes_to_config(config, TYPE_FILAMENT)`
                // (`OrcaSlicer.cpp:3150,587-606`): an empty filament
                // start/end G-code vector is resized to one empty slot.
                insert_default_gcodes(&mut filament_raw, &FILAMENT_GCODE_KEYS);
                resize_empty_filament_gcodes(&mut filament_raw);
                raw.extend_raw(&filament_raw);
                filament_raws.push(filament_raw);
                Ok(merged)
            })
            .collect::<Result<Vec<_>, SliceError>>()?;
        super::cli_metadata::filament_conditions(&mut raw, &filaments);
        let composed =
            compose_resolved_profiles(selection, machine, process, filaments, runtime.resolve());
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
        // Direct CLI files do not resolve preset links (OrcaSlicer.cpp:1953-2010).
        let layers = if self.direct_imports {
            vec![Cow::Borrowed(self.document(position))]
        } else {
            self.resolved_layers(position)?
        };
        let mut fragments = Vec::with_capacity(layers.len());
        for layer in layers {
            let (fragment, project) = ProfileFragment::from_catalog_values(layer.values())?;
            fragments.push(fragment);
            runtime.overlay(project);
            raw.extend(
                layer
                    .fields
                    .iter()
                    .filter(|(key, _)| super::raw::applies(kind, key))
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        let mut merged = if self.direct_imports {
            merge_direct_profile_fragment(&fragments, kind, name)?
        } else {
            merge_profile_fragments(&fragments, kind, name)?
        };
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

/// OrcaSlicer.cpp:588-605, `load_default_gcodes_to_config(TYPE_FILAMENT)`:
/// resize empty start/end G-code vectors to one empty string.
fn resize_empty_filament_gcodes(raw: &mut ProjectSettingsRaw) {
    for key in FILAMENT_GCODE_KEYS {
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
        raw.insert_strings(key, &[String::new()]);
    }
}

/// `option(key, true)` with the option default (`Config.hpp` create path):
/// absent keys are created from the registry default, present keys stay.
/// The pairs are filtered out of the option registry, so every inserted
/// default is a registry `OptionDefinition::default_value`.
fn insert_default_gcodes(raw: &mut ProjectSettingsRaw, keys: &[&str]) {
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
