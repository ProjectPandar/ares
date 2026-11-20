//! The indexed filament merge for a fresh STL with skipped file entries.

use crate::{
    SliceError, filament_options_with_variant, option_definition,
    options::json_deserialize::{decode, deserialize_config_values},
    profiles::composition::ComposedProfile,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

const VARIANT: &str = "filament_extruder_variant";

pub(super) fn merge(
    composed: &mut ComposedProfile,
    raw: &mut ProjectSettingsRaw,
    filament_raws: &[ProjectSettingsRaw],
    (count, loaded): (usize, &[usize]),
) -> Result<(), SliceError> {
    let mut cardinality = vec![1; count];
    for (&slot, source) in loaded.iter().zip(filament_raws) {
        let start: usize = cardinality[..slot].iter().sum();
        let variants = vector(source, VARIANT)?.len().max(1);
        for (key, _) in source.entries() {
            // OrcaSlicer.cpp:3815–3825 only rebuilds these cross-filament options
            // when every slot supplies a configuration.
            if key.starts_with("filament_dev_") || key == "filament_ids" {
                continue;
            }
            merge_option(raw, source, key, (slot, count), (start, variants))?;
        }
        cardinality[slot] = variants;
    }
    if let Some(RawConfigValue::List(dense)) = raw.get("inherits_group") {
        let mut grouped = vec![RawConfigValue::Text(String::new()); count + 2];
        grouped[0] = dense[0].clone();
        grouped[count + 1] = dense[dense.len() - 1].clone();
        for (index, &slot) in loaded.iter().enumerate() {
            grouped[slot + 1] = dense[index + 1].clone();
        }
        raw.insert("inherits_group".to_owned(), RawConfigValue::List(grouped));
    }
    let filament = deserialize_config_values(raw.clone().into_object())
        .map_err(SliceError::InvalidInput)?
        .resolve()
        .filament;
    composed.apply_sparse_filaments(filament, loaded, &cardinality);
    Ok(())
}

fn merge_option(
    raw: &mut ProjectSettingsRaw,
    source: &ProjectSettingsRaw,
    key: &str,
    (slot, count): (usize, usize),
    (start, variants): (usize, usize),
) -> Result<(), SliceError> {
    let Some(definition) = option_definition(key) else {
        return Ok(());
    };
    let mut values = vector(source, key)?;
    let Some(first) = values.first().cloned() else {
        return Ok(());
    };
    let mut destination = vector(raw, key)?;
    if destination.is_empty() {
        destination = if key == VARIANT {
            vec![RawConfigValue::Text("Direct Drive Standard".to_owned()); count]
        } else {
            decode_vector(definition.kind, definition.default_wire_value)?
        };
    }
    let Some(seed) = destination.first().cloned() else {
        return Ok(());
    };
    if filament_options_with_variant().contains(&key) || key == VARIANT {
        // Config.hpp:542–581: replace the target variant interval;
        // growth before that interval uses the first destination value.
        destination.resize(destination.len().max(start + 1), seed);
        values.resize(variants, first);
        destination.splice(start..start + 1, values);
    } else {
        // Config.hpp:451–469: write only this index, growing from
        // the first current value rather than loading a preset.
        destination.resize(destination.len().max(slot + 1), seed);
        destination[slot] = first;
    }
    raw.insert(key.to_owned(), RawConfigValue::List(destination));
    Ok(())
}

fn vector(raw: &ProjectSettingsRaw, key: &str) -> Result<Vec<RawConfigValue>, SliceError> {
    match raw.get(key) {
        Some(RawConfigValue::List(values)) => Ok(values.clone()),
        Some(RawConfigValue::Text(token)) => {
            let definition = option_definition(key).unwrap();
            decode_vector(definition.kind, token)
        }
        _ => Ok(Vec::new()),
    }
}

fn decode_vector(
    kind: crate::OptionValueKind,
    token: &str,
) -> Result<Vec<RawConfigValue>, SliceError> {
    match decode(kind, token).map_err(SliceError::InvalidInput)? {
        RawConfigValue::List(values) => Ok(values),
        _ => Ok(Vec::new()),
    }
}
