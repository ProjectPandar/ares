use super::{
    super::project_refresh::preset::{connection, strings},
    config::{PresetConfig, typed},
};
use crate::{ProfileKind, SliceError, project::raw_settings::RawConfigValue};
use std::collections::BTreeSet;

// PrintConfig.cpp:12139-12265, update_non_diff_values_to_base_config.
pub(super) fn refresh(
    config: &mut PresetConfig,
    base: &PresetConfig,
    protected: &BTreeSet<String>,
    kind: ProfileKind,
) -> Result<(), SliceError> {
    use crate::options::registry as keys;
    let (id_key, variant_key, first, second) = match kind {
        ProfileKind::Machine => (
            "printer_extruder_id",
            "printer_extruder_variant",
            keys::printer_options_with_variant_1(),
            keys::printer_options_with_variant_2(),
        ),
        ProfileKind::Process => (
            "print_extruder_id",
            "print_extruder_variant",
            keys::print_options_with_variant(),
            &[][..],
        ),
        ProfileKind::Filament => (
            "filament_self_index",
            "filament_extruder_variant",
            keys::filament_options_with_variant(),
            &[][..],
        ),
    };
    let current_variants = strings(&config.0, variant_key);
    let base_variants = strings(&base.0, variant_key);
    let current_ids = strings(&config.0, id_key);
    let base_ids = strings(&base.0, id_key);
    let indices = if current_variants.is_empty() {
        map(
            &base_variants,
            &base_ids,
            &[],
            &base_ids[..base_ids.len().min(1)],
        )
    } else if (!current_ids.is_empty() && current_ids.len() != current_variants.len())
        || (!base_ids.is_empty() && base_ids.len() != base_variants.len())
    {
        vec![None; base_variants.len()]
    } else {
        map(&base_variants, &base_ids, &current_variants, &current_ids)
    };
    let mut replacements = Vec::new();
    for (key, value) in config.0.entries() {
        let Some(target) = base.0.get(key) else {
            continue;
        };
        if connection(key) || typed(key, value)?.same(&typed(key, target)?) {
            continue;
        }
        if !protected.contains(key) {
            replacements.push((key.clone(), target.clone()));
            continue;
        }
        let (RawConfigValue::List(source), RawConfigValue::List(target)) = (value, target) else {
            continue;
        };
        let paired = second.contains(&key.as_str());
        if (!paired && !first.contains(&key.as_str()))
            || current_variants.len() > base_variants.len()
        {
            continue;
        }
        if key == id_key || key == variant_key {
            replacements.push((key.clone(), RawConfigValue::List(target.clone())));
            continue;
        }
        let stride = if paired { 2 } else { 1 };
        let size = indices.len() * stride;
        let mut source = source.clone();
        let mut result = target.clone();
        if paired {
            normalize_pair(&mut source, size);
            normalize_pair(&mut result, size);
        } else {
            resize(&mut source, size, target);
            resize(&mut result, size, target);
        }
        for (destination, source_index) in indices.iter().enumerate() {
            let Some(source_index) = source_index else {
                continue;
            };
            for offset in 0..stride {
                result[destination * stride + offset] =
                    source[source_index * stride + offset].clone();
            }
        }
        replacements.push((key.clone(), RawConfigValue::List(result)));
    }
    config.0.extend(replacements);
    Ok(())
}

// PrintConfig.cpp:798-831: exact variant, then first variant of the same id.
fn map(
    variants: &[String],
    ids: &[String],
    from_variants: &[String],
    from_ids: &[String],
) -> Vec<Option<usize>> {
    let count = if variants.is_empty() {
        ids.len()
    } else {
        variants.len()
    };
    let from_count = if from_variants.is_empty() {
        from_ids.len()
    } else {
        from_variants.len()
    };
    (0..count)
        .map(|index| {
            if !ids.is_empty() && index >= ids.len() {
                return None;
            }
            if from_count == 0 {
                return Some(0);
            }
            let same_id = |candidate| {
                ids.is_empty() || from_ids.is_empty() || from_ids.get(candidate) == ids.get(index)
            };
            from_variants
                .iter()
                .enumerate()
                .find(|(candidate, variant)| {
                    Some(*variant) == variants.get(index) && same_id(*candidate)
                })
                .map(|(index, _)| index)
                .or_else(|| (0..from_count).find(|candidate| same_id(*candidate)))
        })
        .collect()
}

// Config.hpp:684-718: extend with the first saved value, or the parent's first value.
fn resize(values: &mut Vec<RawConfigValue>, size: usize, parent: &[RawConfigValue]) {
    if size <= values.len() {
        values.truncate(size);
    } else {
        values.resize(size, values.first().unwrap_or_else(|| &parent[0]).clone());
    }
}

// PrintConfig.cpp:12087-12125: preserve normal/silent pair alignment.
fn normalize_pair(values: &mut Vec<RawConfigValue>, size: usize) {
    if size == 0 {
        values.clear();
        return;
    }
    if values.is_empty() {
        values.resize(size, RawConfigValue::Text("0".to_owned()));
        return;
    }
    let first = values[0].clone();
    let second = values.get(1).unwrap_or(&first).clone();
    if values.len() < 2 {
        values.push(second.clone());
    }
    if !values.len().is_multiple_of(2) {
        values.push(second.clone());
    }
    while values.len() < size {
        values.push(first.clone());
        values.push(second.clone());
    }
    values.truncate(size);
}
