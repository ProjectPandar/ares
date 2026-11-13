//! The filament branch of `PresetBundle::update_compatible` (`PresetBundle.cpp:8517-8673`) with
//! `PresetSelectCompatibleType::Always`: a slot keeps a compatible preset, and a missing or
//! incompatible one takes the best compatible visible preset of the name-sorted collection
//! (`first_compatible_idx`, `Preset.hpp:831-855`).

use std::collections::BTreeSet;

use super::checks::{Candidate, expression_config};
use super::{Catalog, filter_with_catalog};
use crate::SliceError;
use crate::profiles::fragment::ProfileKind;
use crate::project_slice::gcode_emit::value::Value;

/// A filament preset as the match conditions see it.
struct Filament {
    name: String,
    alias: String,
    filament_type: String,
}

/// The filament of each slot after `update_compatible(Always)` for `printer` and `process`;
/// the initialized default where no compatible visible ordinary preset exists. System filaments are visible when
/// `installed` names them (`PresetBundle::load_installed_filaments`), user filaments always.
pub(in crate::profiles) fn select_compatible_filaments(
    catalog: &Catalog,
    (printer, process): (&str, Option<&str>),
    filaments: &[String],
    installed: &[String],
) -> Result<Vec<String>, SliceError> {
    let compatible = filter_with_catalog(catalog, printer, process)?.filaments;
    // PresetBundle.cpp:5419-5436: hardware minimum, padded before compatibility selection.
    let (_, minimum) = catalog.filament_slot_counts(printer)?;
    let mut filaments = filaments.to_vec();
    let installed: BTreeSet<&str> = installed.iter().map(String::as_str).collect();
    let mut candidates = Vec::new();
    for name in &compatible {
        let position = catalog
            .selectable_position(ProfileKind::Filament, name)
            .expect("a compatible filament is in the catalog");
        if catalog.is_user_scope(position) || installed.contains(name.as_str()) {
            candidates.push(filament(catalog, position)?);
        }
    }
    // PresetCollection keeps its presets sorted by name.
    candidates.sort_by(|a, b| a.name.cmp(&b.name));
    if filaments.len() < minimum {
        let last = filaments.last().cloned().unwrap_or_else(|| {
            candidates
                .first()
                .map(|preset| preset.name.clone())
                .unwrap_or_default()
        });
        filaments.resize(minimum, last);
    }
    let printer = catalog
        .selectable_position(ProfileKind::Machine, printer)
        .expect("filter_with_catalog found the printer");
    // `:8636`: `default_filament_profile` of the printer.
    let preferred = texts(expression_config(catalog, printer)?.get("default_filament_profile"));
    let slot = |name: &str| -> Result<Option<Filament>, SliceError> {
        catalog
            .selectable_position(ProfileKind::Filament, name)
            .filter(|position| catalog.missing_parent(*position).is_none())
            .map(|position| filament(catalog, position))
            .transpose()
    };
    if let [name] = filaments.as_slice() {
        // `:8660-8664`: the edited preset after `PresetCollection::update_compatible`
        // (`Preset.hpp:877-885`) with `PreferedFilamentsProfileMatch` (`:8600-8628`).
        if compatible.contains(name)
            || catalog
                .selectable_position(ProfileKind::Filament, name)
                .is_some_and(|position| catalog.is_default(position))
        {
            return Ok(vec![name.clone()]);
        }
        let preset = slot(name)?;
        let (alias, filament_type) = preset.map_or((String::new(), "PLA".to_owned()), |preset| {
            (preset.alias, preset.filament_type)
        });
        return Ok(vec![first_compatible(catalog, &candidates, |candidate| {
            let quality = i32::from(preferred.contains(&candidate.name)) + 1;
            score(candidate, (&alias, &filament_type), quality)
        })]);
    }
    // `:8666-8672`: each missing or incompatible slot with `PreferedFilamentProfileMatch`
    // (`:8574-8598`) and its preferred name, or the first.
    filaments
        .iter()
        .enumerate()
        .map(|(index, name)| {
            if compatible.contains(name)
                || catalog
                    .selectable_position(ProfileKind::Filament, name)
                    .is_some_and(|position| catalog.is_default(position))
            {
                return Ok(name.clone());
            }
            let preset = slot(name)?;
            let (alias, filament_type) = preset.map_or((String::new(), String::new()), |preset| {
                (preset.alias, preset.filament_type)
            });
            let preferred = preferred.get(index).or(preferred.first());
            Ok(first_compatible(catalog, &candidates, |candidate| {
                // Every candidate is visible: + 1. A missing preset's empty type matches none.
                let quality = i32::from(preferred == Some(&candidate.name)) + 2;
                score(candidate, (&alias, &filament_type), quality)
            }))
        })
        .collect()
}

/// The match conditions: the slot preset's alias always wins, else `quality`, ten times for
/// the slot's filament type.
fn score(candidate: &Filament, (alias, filament_type): (&str, &str), quality: i32) -> i32 {
    if !alias.is_empty() && candidate.alias == alias {
        i32::MAX
    } else if candidate.filament_type == filament_type {
        quality * 10
    } else {
        quality
    }
}

/// `first_compatible_idx`: the first candidate of the highest quality.
fn first_compatible(
    catalog: &Catalog,
    candidates: &[Filament],
    quality: impl Fn(&Filament) -> i32,
) -> String {
    let mut best: Option<(&Filament, i32)> = None;
    for candidate in candidates {
        let this = quality(candidate);
        if best.is_none_or(|(_, quality)| this > quality) {
            best = Some((candidate, this));
        }
    }
    best.map(|(candidate, _)| candidate.name.clone())
        .unwrap_or_else(|| {
            catalog
                .document(catalog.default_position(ProfileKind::Filament))
                .name
                .clone()
        })
}

fn filament(catalog: &Catalog, position: usize) -> Result<Filament, SliceError> {
    let candidate = Candidate::load(catalog, position)?;
    // `Preset.cpp:4414-4440`: a user preset's alias is its bare name, without the `@` suffix
    // only for a root preset. A vendor preset's follows `PresetBundle.cpp:7735-7744, 7817-7818`.
    let alias = if catalog.is_user_scope(position) {
        let bare = candidate.name.rsplit('/').next().unwrap_or(&candidate.name);
        match catalog
            .leaf_inherits(position)
            .filter(|parent| !parent.is_empty())
        {
            None => match bare.split('@').next().unwrap_or(bare).trim_end() {
                "" => bare.to_owned(),
                alias => alias.to_owned(),
            },
            Some(_) => bare.to_owned(),
        }
    } else {
        candidate.alias
    };
    // PrintConfigDef: `filament_type` defaults to "PLA".
    let filament_type = texts(expression_config(catalog, position)?.get("filament_type"))
        .into_iter()
        .next()
        .unwrap_or_else(|| "PLA".to_owned());
    Ok(Filament {
        name: candidate.name,
        alias,
        filament_type,
    })
}

fn texts(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::List(values)) => values.iter().flat_map(|value| texts(Some(value))).collect(),
        Some(Value::String(text)) => vec![text.clone()],
        _ => Vec::new(),
    }
}
