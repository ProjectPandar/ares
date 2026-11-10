use std::collections::BTreeSet;

use super::super::keys;
use super::{dev_key, is_vector, replace_variants, vector};
use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) struct Slot<'a> {
    pub(super) index: usize,
    pub(super) filament_count: usize,
    pub(super) range: (usize, usize, usize),
    pub(super) restore: &'a [Option<usize>],
    pub(super) protected: &'a BTreeSet<String>,
    pub(super) refresh_unlisted: bool,
}

pub(super) fn apply(
    result: &mut ProjectSettingsRaw,
    project: &ProjectSettingsRaw,
    config: &ProjectSettingsRaw,
    context: Slot<'_>,
) -> Result<(), SliceError> {
    let Slot {
        index: slot,
        filament_count: count,
        range: (start, old_count, new_count),
        restore,
        protected,
        refresh_unlisted,
    } = context;
    let variant_keys = crate::filament_options_with_variant();
    for (key, source) in config.entries() {
        let kept = if refresh_unlisted {
            protected.contains(key)
        } else {
            project.contains_key(key)
        };
        if let Some((group, length, index)) = match key.as_str() {
            "compatible_printers_condition" => {
                Some(("compatible_machine_expression_group", count + 2, slot + 1))
            }
            "compatible_prints_condition" => {
                Some(("compatible_process_expression_group", count, slot))
            }
            _ => None,
        } {
            if kept {
                continue;
            }
            let mut values = result.get(group).map(vector).unwrap_or_default();
            if values.is_empty() {
                values.resize(length, RawConfigValue::Text(String::new()));
            }
            values[index] = source.clone();
            result.insert(group.to_owned(), RawConfigValue::List(values));
            continue;
        }
        if [
            "compatible_prints",
            "compatible_printers",
            "model_id",
            "dev_model_name",
            "filament_settings_id",
        ]
        .contains(&key.as_str())
            || dev_key(key)
        {
            continue;
        }
        let Some(definition) = crate::option_definition(key) else {
            continue;
        };
        if !is_vector(definition.kind) {
            continue;
        }
        if variant_keys.contains(&key.as_str()) {
            let source = vector(source);
            if source.is_empty() && new_count != 0 {
                return Err(SliceError::InvalidInput(format!(
                    "filament preset has an empty {key} vector"
                )));
            }
            let old = result.get(key).map(vector).unwrap_or_default();
            let restored = if kept { restore } else { &[] };
            let merged = replace_variants(
                old,
                source,
                (restored, !refresh_unlisted || !kept),
                (start, old_count, new_count),
                key,
            )?;
            result.insert(key.clone(), RawConfigValue::List(merged));
        } else if !kept {
            let source = vector(source);
            if source.is_empty() {
                return Err(SliceError::InvalidInput(format!(
                    "filament preset has an empty {key} vector"
                )));
            }
            let mut values = match result.get(key) {
                Some(value) => vector(value),
                None => vector(&keys::default_value(key)?.unwrap()),
            };
            let first = source[0].clone();
            if values.len() <= slot {
                // Config.hpp:451-459: set_at grows with the destination front.
                values.resize(slot + 1, values[0].clone());
            }
            values[slot] = first;
            result.insert(key.clone(), RawConfigValue::List(values));
        }
    }
    Ok(())
}
