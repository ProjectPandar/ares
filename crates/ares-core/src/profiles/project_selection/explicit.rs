//! The plater's config for an explicit selection: `PresetBundle::full_fff_config` over the
//! selected presets (PresetBundle.cpp:5536-5841), with the volume types `load_selections` puts in
//! the project config (4196-4216).

use super::{
    LoadedCatalog, PresetRecord,
    collection::Collection,
    groups::{self, FullFffGroups, Selected},
};
use crate::{
    ProfileKind, ProfileSelection, SliceError, profiles::catalog::ComposedSelection,
    project::raw_settings::RawConfigValue,
};

/// The selected presets composed as full_fff_config. Both upstream routes load the same resolved
/// presets, so the catalog composes their values; full_fff_config decides the keys below and the
/// returned groups, which replace the loaded config's.
pub(in crate::profiles) fn compose(
    catalog: &LoadedCatalog,
    selection: &ProfileSelection,
) -> Result<(ComposedSelection, FullFffGroups), SliceError> {
    let mut composed = catalog.catalog.compose(selection)?;
    let processes = catalog.collection(ProfileKind::Process)?;
    let printers = catalog.collection(ProfileKind::Machine)?;
    let filaments = catalog.collection(ProfileKind::Filament)?;
    let process = record(processes, composed.profile.process_name())?;
    let printer = record(printers, composed.profile.machine_name())?;
    let slots = composed
        .profile
        .filament_names()
        .iter()
        .map(|name| record(filaments, name))
        .collect::<Result<Vec<_>, _>>()?;
    // One filament reads the selected preset's parent (5640-5646); several read each slot's
    // (5684-5708). Each selected preset is its own edited copy.
    let filament_groups = if let [only] = slots.as_slice() {
        vec![selected(filaments, only)?]
    } else {
        slots
            .iter()
            .map(|preset| {
                let parent = groups::filament_parent(preset, slots[0], |name| {
                    filaments.find(name).map(|index| &filaments.records[index])
                });
                Ok(Selected {
                    preset,
                    different: groups::filament_diff(
                        parent.map(|parent| &parent.config),
                        &preset.config,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, SliceError>>()?
    };
    let groups = FullFffGroups::collect(
        &selected(processes, process)?,
        &filament_groups,
        &selected(printers, printer)?,
    );
    let settings = &mut composed.profile.settings;
    // 5636, 5679, 5821: each preset's filament_id, inherited along its chain (7793-7817).
    settings.project.gcode.filament_ids = crate::OrcaStrings(
        slots
            .iter()
            .map(|preset| preset.filament_id.clone())
            .collect(),
    );
    // 4207-4208: with no stored types the project takes the printer's default_nozzle_volume_type.
    settings.project.gcode.nozzle_volume_type = settings
        .printer
        .remaining
        .default_nozzle_volume_type
        .clone();
    let types = composed
        .raw
        .get("default_nozzle_volume_type")
        .cloned()
        .unwrap_or_else(|| {
            RawConfigValue::List(
                crate::option_definition("default_nozzle_volume_type")
                    .unwrap()
                    .default_wire_value
                    .split(',')
                    .map(|value| RawConfigValue::Text(value.to_owned()))
                    .collect(),
            )
        });
    composed.raw.insert("nozzle_volume_type".to_owned(), types);
    Ok((composed, groups))
}

/// A selected preset that is its own edited copy.
fn selected<'a>(
    collection: &'a Collection,
    preset: &'a PresetRecord,
) -> Result<Selected<'a>, SliceError> {
    let parent = groups::selected_parent(preset, preset, &collection.records, |name| {
        collection
            .find(name)
            .map(|index| &collection.records[index])
    });
    Ok(Selected {
        preset,
        different: groups::dirty_options(parent.map(|parent| &parent.config), &preset.config)?,
    })
}

fn record<'a>(collection: &'a Collection, name: &str) -> Result<&'a PresetRecord, SliceError> {
    collection
        .find(name)
        .map(|index| &collection.records[index])
        .ok_or_else(|| SliceError::InvalidInput(format!("preset '{name}' is not loaded")))
}
