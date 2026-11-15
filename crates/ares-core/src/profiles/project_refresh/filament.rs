use super::{keys, preset};
mod slot;

use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn apply(
    result: &mut ProjectSettingsRaw,
    project: &ProjectSettingsRaw,
    filaments: &[Option<ProjectSettingsRaw>],
    dirty: &[String],
    refresh_unlisted: bool,
) -> Result<(), SliceError> {
    if filaments.iter().all(Option::is_none) {
        return Ok(());
    }
    let count = filaments.len();
    let last_id = i32::try_from(count).unwrap();
    let mut ids = preset::strings(project, "filament_self_index")
        .iter()
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|error| SliceError::InvalidInput(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // OrcaSlicer.cpp:3534–3547 regenerates stale indices before saving old ranges.
    if ids.len() < count
        || ids.iter().any(|id| *id < 1)
        || ids.iter().copied().max().unwrap_or(0) != last_id
    {
        ids = (1..=last_id).collect();
    }
    let mut groups = Vec::new();
    for id in &ids {
        if groups
            .last()
            .is_some_and(|(previous, _): &(i32, usize)| previous == id)
        {
            groups.last_mut().unwrap().1 += 1;
        } else {
            groups.push((*id, 1));
        }
    }
    groups.resize(count, (0, 0));
    normalize_variants(result, filaments)?;
    let mut start = 0;
    for (slot, config) in filaments.iter().enumerate() {
        let old_count = groups[slot].1;
        let Some(config) = config else {
            start += old_count;
            continue;
        };
        let protected = preset::dirty_keys(dirty, slot + 1)?;
        let variants = vector(config.get("filament_extruder_variant").unwrap());
        let old_variants = result
            .get("filament_extruder_variant")
            .map(vector)
            .unwrap_or_default();
        let restore = variants
            .iter()
            .map(|variant| {
                (start..start + old_count).find(|index| old_variants.get(*index) == Some(variant))
            })
            .collect::<Vec<_>>();
        slot::apply(
            result,
            project,
            config,
            slot::Slot {
                index: slot,
                filament_count: count,
                range: (start, old_count, variants.len()),
                restore: &restore,
                protected: &protected,
                refresh_unlisted,
            },
        )?;
        groups[slot].1 = variants.len();
        start += variants.len();
    }
    let self_index = groups
        .iter()
        .enumerate()
        .flat_map(|(slot, (_, length))| {
            std::iter::repeat_n(RawConfigValue::Text((slot + 1).to_string()), *length)
        })
        .collect();
    result.insert(
        "filament_self_index".to_owned(),
        RawConfigValue::List(self_index),
    );
    if refresh_unlisted && filaments.iter().any(Option::is_none) {
        return Ok(());
    }
    set_filament_dev_options(
        result,
        filaments.iter().map(Option::as_ref),
        (!refresh_unlisted).then_some(project),
    )?;
    Ok(())
}

// PrintConfig.cpp:11430-11443; CLI:3815-3834 preserves original project keys on refresh.
pub(crate) fn set_filament_dev_options<'a>(
    result: &mut ProjectSettingsRaw,
    filaments: impl Iterator<Item = Option<&'a ProjectSettingsRaw>> + Clone,
    preserved: Option<&ProjectSettingsRaw>,
) -> Result<(), SliceError> {
    for key in DEV_KEYS {
        if preserved.is_some_and(|project| project.contains_key(key))
            || !filaments
                .clone()
                .flatten()
                .any(|config| config.contains_key(key))
        {
            continue;
        }
        let mut values = Vec::new();
        for config in filaments.clone() {
            let default;
            let source = if let Some(value) = config.and_then(|raw| raw.get(key)) {
                value
            } else {
                default = keys::default_value(key)?.unwrap();
                &default
            };
            // Config.cpp:985-987 routes scalar strings to the option decoder;
            // Config.hpp:1225-1229 decodes coStrings wire into a vector.
            let decoded = match source {
                RawConfigValue::Text(wire) => Some(crate::options::json_deserialize::decode(
                    crate::option_definition(key).unwrap().kind,
                    wire,
                )),
                _ => None,
            }
            .transpose()
            .map_err(|error| {
                SliceError::InvalidInput(format!("invalid Orca option {key}: {error}"))
            })?;
            let source = decoded.as_ref().unwrap_or(source);
            let RawConfigValue::List(source) = source else {
                return Err(SliceError::InvalidInput(format!(
                    "invalid Orca option {key}: expected a vector"
                )));
            };
            values.extend_from_slice(source);
        }
        result.insert((*key).to_owned(), RawConfigValue::List(values));
    }
    Ok(())
}

pub(crate) fn loaded_filament_ordinary_values(
    filaments: &[ProjectSettingsRaw],
) -> Result<ProjectSettingsRaw, SliceError> {
    use std::collections::{BTreeMap, btree_map::Entry};

    let mut slots: BTreeMap<String, Vec<RawConfigValue>> = BTreeMap::new();
    for (slot, config) in filaments.iter().enumerate() {
        for (key, source) in config.entries() {
            if SKIPPED_BY_SET_AT.contains(&key.as_str())
                || dev_key(key)
                || crate::filament_options_with_variant().contains(&key.as_str())
                || !crate::option_definition(key)
                    .is_some_and(|definition| is_vector(definition.kind))
            {
                continue;
            }
            let first = source.element(0).ok_or_else(|| {
                SliceError::InvalidInput(
                    "ConfigOptionVector::set_at(): Assigning from an empty vector".to_owned(),
                )
            })?;
            let values = match slots.entry(key.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(vector(&keys::default_value(key)?.unwrap())),
            };
            // OrcaSlicer.cpp:3770–3771; Config.hpp:450–473.
            if values.len() <= slot {
                values.resize(slot + 1, values[0].clone());
            }
            values[slot] = first.clone();
        }
    }
    let mut raw = ProjectSettingsRaw::default();
    raw.extend(
        slots
            .into_iter()
            .map(|(key, values)| (key, RawConfigValue::List(values))),
    );
    Ok(raw)
}

const SKIPPED_BY_SET_AT: &[&str] = &[
    "compatible_prints",
    "compatible_printers",
    "model_id",
    "dev_model_name",
    "filament_settings_id",
];

fn normalize_variants(
    result: &mut ProjectSettingsRaw,
    filaments: &[Option<ProjectSettingsRaw>],
) -> Result<(), SliceError> {
    for key in crate::filament_options_with_variant() {
        if filaments
            .iter()
            .flatten()
            .any(|config| config.contains_key(key))
            && !result.contains_key(key)
            && let Some(value) = keys::default_value(key)?
        {
            result.insert((*key).to_owned(), value);
        }
    }
    result.normalize_filament_values_to_variants();
    Ok(())
}

fn replace_variants(
    mut old: Vec<RawConfigValue>,
    mut source: Vec<RawConfigValue>,
    (restore, pad): (&[Option<usize>], bool),
    (start, old_count, new_count): (usize, usize, usize),
    key: &str,
) -> Result<Vec<RawConfigValue>, SliceError> {
    if source.len() != new_count {
        if !pad {
            return Err(SliceError::InvalidInput(format!(
                "filament preset {key} has {} values for {new_count} variants",
                source.len()
            )));
        }
        source.resize(new_count, source[0].clone());
    }
    if old.len() < start + old_count {
        // `ConfigOptionVector::resize` (`Config.hpp:698-715`) extends with its own front value.
        let pad = old.first().unwrap_or(&source[0]).clone();
        old.resize(start + old_count, pad);
    }
    let inserted = source
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            restore
                .get(index)
                .copied()
                .flatten()
                .and_then(|restore| old.get(restore))
                .cloned()
                .unwrap_or(value)
        })
        .collect::<Vec<_>>();
    old.splice(start..start + old_count, inserted);
    Ok(old)
}

fn vector(value: &RawConfigValue) -> Vec<RawConfigValue> {
    match value {
        RawConfigValue::List(values) => values.clone(),
        _ => unreachable!("preset vectors have been normalized before application"),
    }
}

pub(in crate::profiles) fn is_vector(kind: crate::OptionValueKind) -> bool {
    use crate::OptionValueKind as K;
    matches!(
        kind,
        K::Bools
            | K::BoolsNullable
            | K::Enums
            | K::EnumsNullable
            | K::Floats
            | K::FloatsNullable
            | K::FloatsOrPercents
            | K::Ints
            | K::IntsNullable
            | K::Percents
            | K::PercentsNullable
            | K::Strings
            | K::Points
            | K::PointsGroups
    )
}

pub(crate) fn dev_key(key: &str) -> bool {
    DEV_KEYS.contains(&key)
}

const DEV_KEYS: &[&str] = &[
    "filament_dev_ams_drying_ams_limitations",
    "filament_dev_ams_drying_temperature",
    "filament_dev_ams_drying_time",
    "filament_dev_ams_drying_heat_distortion_temperature",
    "filament_dev_chamber_drying_bed_temperature",
    "filament_dev_chamber_drying_time",
    "filament_dev_drying_softening_temperature",
    "filament_dev_drying_cooling_temperature",
];
