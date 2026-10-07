//! `OrcaSlicer.cpp:3340–3503` CLI preparation before `Print::apply` (8500fcd).
mod context;
pub(crate) use context::PreparationContext;

use crate::{
    ExtruderType, NozzleVolumeType, Nullable, OrcaFloat, ProjectSettings, SliceError,
    project::raw_settings::ProjectSettingsRaw,
};

use super::flush_volume::{calculate, rgba};

pub(crate) fn prepare(
    settings: &mut ProjectSettings,
    raw: &ProjectSettingsRaw,
    context: &PreparationContext,
    overrides: &crate::ConfigOverrides,
) -> Result<(), SliceError> {
    let nozzle_count = settings.project.print.nozzle_diameter.0.len();
    let colors = &settings.filament.gcode.filament_colour.0;
    // Byte/project entry corresponds to CLI --slice without extra nozzle/color
    // arguments: new types default to Standard per nozzle (3022–3047).
    // Retain vector shape: one Standard differs from two Standards. A malformed
    // supplied matrix alone is NOT a rebuild trigger (3351).
    let types = &settings.project.gcode.nozzle_volume_type.0;
    let new_types = if overrides.contains_key("nozzle_volume_type") {
        types.clone()
    } else {
        vec![NozzleVolumeType::Standard; nozzle_count]
    };
    let changed = context.colour_override
        || context.current_nozzle_count != nozzle_count
        || context.current_volume_types != new_types;
    // CLI:3340–3351 requires a project colour option, not the typed default
    // supplied later by FullPrintConfig. Sparse STL profiles may omit it.
    if !raw.contains_key("filament_colour")
        || colors.is_empty()
        || (raw.contains_key("flush_volumes_matrix") && !changed)
    {
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
    if requires_dataset && !covered {
        return Ok(());
    }
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
    let mut matrix = vec![0.0; count * count * nozzle_count];
    for (nozzle, segment) in matrix.chunks_exact_mut(count * count).enumerate() {
        let dataset = requires_dataset
            .then(|| dataset(settings, raw, nozzle))
            .transpose()?;
        for (index, value) in segment.iter_mut().enumerate() {
            let (from, to) = (index / count, index % count);
            *value = f64::from(match (from == to, support[from].0, support[to].0) {
                (true, _, _) => 0,
                (_, _, true) => 230,
                (_, true, _) => {
                    calculate(minima[from], dataset.unwrap(), colors[from], colors[to]).max(700)
                }
                (_, false, false) => {
                    calculate(minima[from], dataset.unwrap(), colors[from], colors[to])
                }
            });
        }
    }
    settings.project.print.flush_volumes_matrix.0 = matrix;
    Ok(())
}

fn at<T>(values: &[T], index: usize) -> &T {
    values.get(index).unwrap_or(&values[0])
}

fn dataset(
    settings: &ProjectSettings,
    raw: &ProjectSettingsRaw,
    nozzle: usize,
) -> Result<i32, SliceError> {
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
        // PrintConfig.cpp:9004–9060, called here by CLI:3457–3462.
        let complete_ids =
            printer.printer_extruder_id.0.len() >= printer.printer_extruder_variant.0.len();
        let id_for_variant = |index: usize| {
            if complete_ids {
                printer.printer_extruder_id.0[index].0
            } else {
                generated_printer_id(settings, raw, index)
            }
        };
        printer
            .printer_extruder_variant
            .0
            .iter()
            .enumerate()
            .find_map(|(i, token)| {
                (token == &variant && id_for_variant(i) == (nozzle + 1) as i32).then_some(i)
            })
            .ok_or_else(|| {
                SliceError::InvalidInput("invalid Orca option printer_extruder_variant".into())
            })?
    } else {
        0
    };
    Ok(match values[index] {
        Nullable::Value(value) => value.0,
        Nullable::Nil => i32::MAX,
    })
}

fn generated_printer_id(settings: &ProjectSettings, raw: &ProjectSettingsRaw, index: usize) -> i32 {
    if !raw.contains_key("extruder_variant_list") {
        return 0;
    }
    settings
        .printer
        .remaining
        .extruder_variant_list
        .0
        .iter()
        .enumerate()
        .flat_map(|(physical, tokens)| {
            tokens
                .split(',')
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(move |_| (physical + 1) as i32)
        })
        .nth(index)
        .unwrap_or(0)
}

// Pure extraction of Plater.cpp:764–835. CLI deliberately calls this for
// nozzle 0 before iterating all nozzles, not once per nozzle (3412).
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
