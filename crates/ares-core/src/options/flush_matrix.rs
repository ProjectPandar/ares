//! `OrcaSlicer.cpp:3851–4023` CLI preparation before `Print::apply`
//! (345efec64c7b95a05ce5cf96df0dde66a158835c).
mod context;
pub(crate) use context::PreparationContext;

use serde::Deserialize;

use crate::{
    ExtruderType, NozzleVolumeType, Nullable, OrcaFloat, OrcaStrings, ProjectSettings, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

use super::flush_volume::{calculate, rgba};

#[derive(Deserialize)]
struct LoadedPalette {
    filament_colour: OrcaStrings,
}

pub(crate) fn prepare(
    settings: &mut ProjectSettings,
    raw: &ProjectSettingsRaw,
    context: &PreparationContext,
    overrides: &crate::ConfigOverrides,
) -> Result<(), SliceError> {
    let nozzle_count = settings.project.print.nozzle_diameter.0.len();
    // CLI:1893,2528-2529: count selected presets, not diameter overrides.
    let filament_count = settings.project.preset.filament_settings_id.0.len();
    let has_palette = raw.contains_key("filament_colour");
    let mut palette_raw = ProjectSettingsRaw::default();
    // Config.cpp:323–341,1721–1741: option(key, true) clones the registered
    // default before CLI:3862 grows it; a later typed palette is not that option.
    let value = match raw.get("filament_colour") {
        Some(value) => value.clone(),
        None => RawConfigValue::Text(
            super::registry::option_definition("filament_colour")
                .unwrap()
                .default_wire_value
                .to_owned(),
        ),
    };
    palette_raw.insert("filament_colour".to_owned(), value);
    let LoadedPalette {
        mut filament_colour,
    } = super::json_deserialize::deserialize_preset_config(palette_raw.into_object()).map_err(
        |error| SliceError::InvalidInput(format!("invalid project settings JSON: {error}")),
    )?;
    let grown = filament_count > 0 && (!has_palette || filament_colour.0.len() < filament_count);
    if !has_palette && !grown {
        return Ok(());
    }
    if grown {
        filament_colour
            .0
            .resize(filament_count, "#FFFFFF".to_owned());
    }
    settings.filament.gcode.filament_colour = filament_colour;
    let colors = &settings.filament.gcode.filament_colour.0;
    // CLI:3415–3441 defaults new types to Standard per nozzle. Retain vector
    // shape: one Standard differs from two Standards. A malformed supplied
    // matrix alone is NOT a rebuild trigger (3866).
    let types = &settings.project.gcode.nozzle_volume_type.0;
    let new_types = if overrides.contains_key("nozzle_volume_type") {
        types.clone()
    } else {
        vec![NozzleVolumeType::Standard; nozzle_count]
    };
    let changed = grown
        || context.colour_override
        || context.current_nozzle_count != nozzle_count
        || context.current_volume_types != new_types;
    if colors.is_empty() || (raw.contains_key("flush_volumes_matrix") && !changed) {
        return Ok(());
    }
    let support = &settings.filament.gcode.filament_is_support.0;
    let requires_dataset = support.first().is_some_and(|first| {
        (0..colors.len()).any(|from| {
            (0..colors.len()).any(|to| from != to && !support.get(to).unwrap_or(first).0)
        })
    });
    let covered = types.len() >= nozzle_count
        && settings.printer.gcode.printer_extruder_variant.0.len() >= nozzle_count
        && settings.printer.gcode.printer_extruder_id.0.len() >= nozzle_count;
    settings
        .project
        .print
        .flush_multiplier
        .0
        .resize(nozzle_count, OrcaFloat(1.0));
    let datasets = if requires_dataset {
        if !covered {
            return Ok(());
        }
        let Some(datasets) = (0..nozzle_count)
            .map(|nozzle| dataset(settings, raw, nozzle))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(());
        };
        Some(datasets)
    } else {
        None
    };
    if support.len() != colors.len() {
        return Err(SliceError::InvalidInput(format!(
            "filament_is_support's count {} not equal to filament_colour's size {}",
            support.len(),
            colors.len()
        )));
    }
    let minima = minimum_volumes(settings, raw);
    let colors: Vec<_> = colors.iter().map(|color| rgba(color)).collect();
    let count = colors.len();
    let matrix = &mut settings.project.print.flush_volumes_matrix.0;
    // CLI:3944–3949 resizes the applied matrix before computing its cells.
    matrix.resize(count * count * nozzle_count, 0.0);
    for (nozzle, segment) in matrix.chunks_exact_mut(count * count).enumerate() {
        let Some(dataset) = datasets.as_ref().map(|datasets| datasets[nozzle]) else {
            segment.fill(230.0);
            segment
                .iter_mut()
                .step_by(count + 1)
                .for_each(|value| *value = 0.0);
            continue;
        };
        for (index, value) in segment.iter_mut().enumerate() {
            let (from, to) = (index / count, index % count);
            *value = f64::from(match (from == to, support[from].0, support[to].0) {
                (true, _, _) => 0,
                (_, _, true) => 230,
                (_, true, _) => calculate(minima[from], dataset, colors[from], colors[to]).max(700),
                (_, false, false) => calculate(minima[from], dataset, colors[from], colors[to]),
            });
        }
    }
    Ok(())
}

fn at<T>(values: &[T], index: usize) -> &T {
    values.get(index).unwrap_or(&values[0])
}

fn dataset(settings: &ProjectSettings, raw: &ProjectSettingsRaw, nozzle: usize) -> Option<i32> {
    let printer = &settings.printer.gcode;
    let mut values = printer.nozzle_flush_dataset.0.clone();
    let has_variants = raw.contains_key("printer_extruder_variant");
    values.resize(
        if has_variants {
            printer.printer_extruder_variant.0.len()
        } else {
            1
        },
        Nullable::Value(crate::OrcaInt(0)),
    );
    let index = if has_variants && raw.contains_key("printer_extruder_id") {
        let extruder = match at(&printer.extruder_type.0, nozzle) {
            ExtruderType::DirectDrive => "Direct Drive",
            ExtruderType::Bowden => "Bowden",
        };
        let volume = match at(&settings.project.gcode.nozzle_volume_type.0, nozzle) {
            NozzleVolumeType::Standard => "Standard",
            NozzleVolumeType::HighFlow => "High Flow",
        };
        let variant = format!("{extruder} {volume}");
        let groups = if raw.contains_key("extruder_variant_list") {
            settings
                .printer
                .remaining
                .extruder_variant_list
                .0
                .as_slice()
        } else {
            &[]
        };
        super::project_variants::get_index_for_extruder(
            ((nozzle + 1) as i32, &variant),
            Some(&printer.printer_extruder_id.0),
            &printer.printer_extruder_variant.0,
            groups,
            1,
        )?
    } else {
        0
    };
    Some(match values[index] {
        Nullable::Value(value) => value.0,
        Nullable::Nil => i32::MAX,
    })
}

// Pure extraction of Plater.cpp:1251–1314. CLI deliberately calls this for
// nozzle 0 before iterating all nozzles, not once per nozzle (3927).
fn minimum_volumes(settings: &ProjectSettings, raw: &ProjectSettingsRaw) -> Vec<i32> {
    let printer = &settings.printer.gcode;
    let nozzle_volume = match settings.printer.remaining.nozzle_volume.0[0] {
        Nullable::Value(value) => value.0 as i32,
        Nullable::Nil => 0,
    };
    let level = printer.enable_long_retraction_when_cut.0;
    let distance = printer.retraction_distances_when_cut.0[0].0 as i32;
    let active = printer.long_retractions_when_cut.0[0].0;
    let overrides = &settings.filament.retract_overrides;
    (0..settings.filament.gcode.filament_diameter.0.len())
        .map(|i| {
            let mut retract = if level != 0 && active { distance } else { 0 };
            let filament_active = if raw.contains_key("filament_long_retractions_when_cut") {
                match at(&overrides.filament_long_retractions_when_cut, i) {
                    Nullable::Value(value) => u8::from(value.0),
                    Nullable::Nil => u8::MAX,
                }
            } else {
                0
            };
            if filament_active == 0 {
                retract = 0;
            } else if filament_active == 1 && level == 2 {
                retract = match at(&overrides.filament_retraction_distances_when_cut, i) {
                    Nullable::Value(value) => value.0 as i32,
                    Nullable::Nil => distance,
                };
            }
            (f64::from(nozzle_volume)
                - std::f64::consts::PI * 1.75 * 1.75 / 4.0 * f64::from(retract)) as i32
        })
        .collect()
}
