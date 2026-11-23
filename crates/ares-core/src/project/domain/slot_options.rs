use super::Project;
use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};
use crate::{FilamentSlot, FilamentSlotState, ProjectSettings, SliceError, plater::SlotChange};

const MIXED_FIELDS: [(&str, &str); 7] = [
    ("filament_is_mixed", "0"),
    ("filament_mixed_components", ""),
    ("filament_mixed_sublayer_ratios", ""),
    ("filament_mixed_gradient", "0"),
    ("filament_mixed_gradient_range", ""),
    ("filament_mixed_gradient_curve", ""),
    ("filament_mixed_gradient_per_part", "0"),
];
const MAP_FIELDS: [(&str, &str); 3] = [
    ("filament_map", "1"),
    ("filament_nozzle_map", "0"),
    ("filament_volume_map", "0"),
];

fn set_values(raw: &mut ProjectSettingsRaw, key: &str, values: Vec<RawConfigValue>) {
    raw.insert(key.to_owned(), RawConfigValue::List(values));
}

fn text(value: impl ToString) -> RawConfigValue {
    RawConfigValue::Text(value.to_string())
}

pub(crate) struct SlotOptionsSnapshot {
    raw: ProjectSettingsRaw,
    settings: ProjectSettings,
    plates: Vec<crate::project::model_settings::PlateSettings>,
}

impl Project {
    pub(crate) fn replay_slot_options(
        &mut self,
        edits: &crate::PlaterEdits,
    ) -> Result<(), SliceError> {
        for &change in &edits.slot_changes {
            self.apply_slot_options(change)?;
        }
        Ok(())
    }

    pub(crate) fn filament_slots(&self) -> Vec<FilamentSlot> {
        let settings = self.settings();
        let names = &settings.project.preset.filament_settings_id.0;
        let flags = self.documents.project_settings_raw.get("filament_is_mixed");
        settings
            .filament
            .gcode
            .filament_colour
            .0
            .iter()
            .enumerate()
            .map(|(index, colour)| {
                let is_mixed = flags
                    .and_then(|flags| flags.element(index))
                    .and_then(RawConfigValue::token)
                    .is_some_and(|flag| matches!(flag.as_str(), "1" | "true"));
                FilamentSlot {
                    preset: names.get(index).cloned().unwrap_or_default(),
                    colour: colour.clone(),
                    is_mixed,
                }
            })
            .collect()
    }

    pub(crate) fn slot_options_snapshot(&self) -> SlotOptionsSnapshot {
        SlotOptionsSnapshot {
            raw: self.documents.project_settings_raw.clone(),
            settings: self.settings.clone(),
            plates: self.documents.model_settings.plates.clone(),
        }
    }

    pub(crate) fn restore_slot_options(&mut self, options: &SlotOptionsSnapshot) {
        self.documents.project_settings_raw.clone_from(&options.raw);
        self.settings.clone_from(&options.settings);
        self.documents
            .model_settings
            .plates
            .clone_from(&options.plates);
    }

    pub(crate) fn apply_slot_options(&mut self, change: SlotChange) -> Result<(), SliceError> {
        let slots = self.filament_slots();
        let count = match change {
            SlotChange::Insert { .. } => slots.len() + 1,
            SlotChange::Remove { .. } => slots.len() - 1,
        };
        let mut raw = self.documents.project_settings_raw.clone();
        let nozzles = raw
            .get("nozzle_diameter")
            .map_or(1, RawConfigValue::len)
            .max(1);
        let print = &self.settings.project.print;
        let mut flush = FilamentSlotState {
            flush_multiplier: print
                .flush_multiplier
                .0
                .iter()
                .map(|value| value.0)
                .collect(),
            flush_volumes_vector: print
                .flush_volumes_vector
                .0
                .iter()
                .map(|value| value.0)
                .collect(),
            flush_volumes_matrix: print.flush_volumes_matrix.0.clone(),
            ..Default::default()
        };
        if let SlotChange::Insert { at, .. } = change {
            let first = flush
                .flush_volumes_vector
                .first_chunk::<2>()
                .copied()
                .unwrap_or([140.0; 2]);
            let pairs = &mut flush.flush_volumes_vector;
            if pairs.len() == 2 * slots.len() {
                let index = 2 * (at as usize - 1);
                pairs.splice(index..index, first);
            }
        }
        flush.resize_flush(count, nozzles, |index| change.previous_index(index))?;
        for (key, values) in [
            ("flush_multiplier", flush.flush_multiplier),
            ("flush_volumes_vector", flush.flush_volumes_vector),
            ("flush_volumes_matrix", flush.flush_volumes_matrix),
        ] {
            set_values(&mut raw, key, values.into_iter().map(text).collect());
        }
        let last = slots
            .last()
            .map(|slot| slot.preset.as_str())
            .unwrap_or_default();
        let names = (0..count)
            .map(|index| {
                text(
                    change
                        .previous_index(index)
                        .map_or(last, |old| slots[old].preset.as_str()),
                )
            })
            .collect();
        set_values(&mut raw, "filament_settings_id", names);
        resize_option_vectors(&mut raw, (&slots, count), change);
        for key in ["support_filament", "support_interface_filament"] {
            if let Some(slot) = raw
                .get(key)
                .and_then(RawConfigValue::token)
                .and_then(|value| value.parse::<i32>().ok())
            {
                if let Some(new) = change.surviving(slot) {
                    raw.insert(key.to_owned(), text(new));
                } else {
                    raw.remove(key);
                }
            }
        }
        let ranges = raw
            .get("other_layers_print_sequence_nums")
            .and_then(RawConfigValue::token)
            .and_then(|value| value.parse().ok())
            .unwrap_or_default();
        for key in ["first_layer_print_sequence", "other_layers_print_sequence"] {
            if let Some(RawConfigValue::List(values)) = raw.get(key)
                && let Some(ids) = values
                    .iter()
                    .map(|value| value.token()?.parse::<i32>().ok())
                    .collect::<Option<Vec<_>>>()
                && let Some(ids) =
                    super::slot_references::remap_print_order(key, &ids, ranges, change)
            {
                set_values(&mut raw, key, ids.into_iter().map(text).collect());
            }
        }
        let mut typed = raw.clone();
        typed.complete_filament_variant_defaults()?;
        self.settings =
            crate::options::json_deserialize::deserialize_refreshed_config(typed.into_object())
                .map_err(|error| {
                    SliceError::InvalidInput(format!("invalid filament slot settings: {error}"))
                })?
                .resolve();
        if let SlotChange::Insert { at, .. } = change {
            let mixed: Vec<_> = (0..count)
                .map(|index| {
                    change
                        .previous_index(index)
                        .is_some_and(|old| slots[old].is_mixed)
                })
                .collect();
            crate::options::flush_matrix::initialize_added_slot(
                &mut self.settings,
                &raw,
                at as usize - 1,
                &mixed,
            );
            set_values(
                &mut raw,
                "flush_volumes_matrix",
                self.settings
                    .project
                    .print
                    .flush_volumes_matrix
                    .0
                    .iter()
                    .copied()
                    .map(text)
                    .collect(),
            );
        }
        self.documents.project_settings_raw = raw;
        Ok(())
    }
}

fn resize_option_vectors(
    raw: &mut ProjectSettingsRaw,
    (slots, count): (&[FilamentSlot], usize),
    change: SlotChange,
) {
    let colours: Vec<_> = (0..count)
        .map(|index| match change.previous_index(index) {
            Some(old) => text(&slots[old].colour),
            None => match change {
                SlotChange::Insert { colour, .. } => text(colour),
                _ => unreachable!(),
            },
        })
        .collect();
    for (key, default) in MAP_FIELDS
        .into_iter()
        .chain(MIXED_FIELDS)
        .chain([("filament_multi_colour", ""), ("filament_colour_type", "1")])
    {
        let original = raw.get(key);
        let values = (0..count)
            .map(|index| {
                let old = change.previous_index(index);
                if key == "filament_multi_colour" && matches!(change, SlotChange::Insert { .. }) {
                    return colours[index].clone();
                }
                let value = old
                    .and_then(|old| original?.element(old))
                    .cloned()
                    .unwrap_or_else(|| text(default));
                if key == "filament_mixed_components"
                    && matches!(change, SlotChange::Remove { .. })
                    && let RawConfigValue::Text(components) = &value
                    && !components.is_empty()
                {
                    return text(
                        components
                            .split(',')
                            .map(|item| {
                                item.trim()
                                    .parse::<i32>()
                                    .map(|slot| change.surviving(slot).unwrap_or(0).to_string())
                                    .unwrap_or_else(|_| item.to_owned())
                            })
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                }
                value
            })
            .collect();
        set_values(raw, key, values);
    }
    for &key in crate::filament_option_keys() {
        if matches!(
            key,
            "default_filament_profile"
                | "filament_settings_id"
                | "filament_colour"
                | "filament_multi_colour"
                | "filament_colour_type"
        ) || MAP_FIELDS
            .iter()
            .chain(MIXED_FIELDS.iter())
            .any(|(configured, _)| *configured == key)
        {
            continue;
        }
        let Some(RawConfigValue::List(values)) = raw.get(key) else {
            continue;
        };
        if values.is_empty() {
            continue;
        }
        let values = (0..count)
            .map(|index| {
                let old = change.previous_index(index).unwrap_or(slots.len() - 1);
                values.get(old).unwrap_or(&values[0]).clone()
            })
            .collect();
        set_values(raw, key, values);
    }
    set_values(raw, "filament_colour", colours);
}
