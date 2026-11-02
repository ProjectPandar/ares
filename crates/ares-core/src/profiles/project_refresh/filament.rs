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
    let mut ids = preset::strings(project, "filament_self_index")
        .iter()
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| SliceError::InvalidInput(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for index in ids.len()..count {
        ids.push(index + 1);
    }
    let mut groups = Vec::new();
    for id in &ids {
        if groups
            .last()
            .is_some_and(|(previous, _): &(usize, usize)| previous == id)
        {
            groups.last_mut().unwrap().1 += 1;
        } else {
            groups.push((*id, 1));
        }
    }
    groups.resize(count, (0, 0));
    normalize_variants(result, count, &ids, filaments)?;
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
    for key in DEV_KEYS {
        if !refresh_unlisted && project.contains_key(key) {
            continue;
        }
        if refresh_unlisted && filaments.iter().any(Option::is_none) {
            continue;
        }
        let mut values = Vec::new();
        for config in filaments {
            let default;
            let source = if let Some(value) = config.as_ref().and_then(|raw| raw.get(key)) {
                value
            } else {
                default = keys::default_value(key)?.unwrap();
                &default
            };
            values.extend(vector(source));
        }
        result.insert((*key).to_owned(), RawConfigValue::List(values));
    }
    Ok(())
}

fn normalize_variants(
    result: &mut ProjectSettingsRaw,
    count: usize,
    ids: &[usize],
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
        let Some(value) = result.get(key) else {
            continue;
        };
        let values = vector(value);
        if ids.len() <= count || (values.len() != count && values.len() != 1) {
            continue;
        }
        // PrintConfig.cpp:11431–11457: one value per filament or singleton -> original variants.
        let expanded = ids
            .iter()
            .map(|id| values[if values.len() == 1 { 0 } else { id - 1 }].clone())
            .collect();
        result.insert((*key).to_owned(), RawConfigValue::List(expanded));
    }
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
        old.resize(start + old_count, source[0].clone());
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

fn is_vector(kind: crate::OptionValueKind) -> bool {
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

fn dev_key(key: &str) -> bool {
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
