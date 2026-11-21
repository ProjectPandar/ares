//! Print.cpp:1743–1794,1839–1947,2226–2237: applied nozzle compatibility.
mod material;

use crate::{
    ProcessPrintSequence, Project, ProjectSettings, SliceError,
    project::effective_config::{
        types::BoundedResolvedProjectConfig,
        usage::{append_object_filaments, collect_usage, object_usage_input, supported_brim},
    },
};

const INCOMPATIBLE: &str = "Selected nozzle temperatures are incompatible. Each filament's nozzle temperature must fall within the recommended nozzle temperature range of the other filaments. Otherwise, nozzle clogging or printer damage may occur. If you still want to print, you can enable the option in Preferences / Control / Slicing / Remove mixed temperature restriction.";
const INVALID_RANGE: &str = "Invalid recommended nozzle temperature range. The lower bound must be lower than the upper bound.";

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate: u32,
    allow_mix_temp: bool,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    if settings.project.print.nozzle_diameter.0.len() >= 2
        || resolved.usage.supported_used_filaments.len() < 2
    {
        return Ok(());
    }
    let objects = resolved
        .objects
        .iter()
        .map(|object| object_usage_input(&project.objects()[object.source_object_index], object))
        .collect::<Vec<_>>();
    let filament_count = settings.filament.gcode.filament_diameter.0.len();
    // Print::extruders() excludes layer-slider ToolChanges unless requested.
    let usage = collect_usage(&objects, settings, filament_count, &[]);
    if usage.supported_used_filaments.len() < 2 {
        return Ok(());
    }

    let compatibility = if settings.process.print.print_sequence == ProcessPrintSequence::ByObject {
        let has_brim = objects.iter().any(|input| supported_brim(input.object));
        let mut used = Vec::new();
        objects
            .iter()
            .find_map(|input| {
                used.clear();
                append_object_filaments(&mut used, input, has_brim);
                let object = input.object;
                if object.enable_support.0
                    || object.enforce_support_layers.0 > 0
                    || object.raft_layers.0 > 0
                {
                    used.extend(
                        [
                            object.support_filament.0,
                            object.support_interface_filament.0,
                        ]
                        .into_iter()
                        .filter_map(|selector| usize::try_from(selector - 1).ok())
                        .filter(|&slot| slot < filament_count),
                    );
                }
                used.sort_unstable();
                used.dedup();
                check(settings, &used).err()
            })
            .map_or(Ok(()), Err)
    } else {
        check(settings, &usage.supported_used_filaments)
    };
    let message = match compatibility {
        Ok(()) => return Ok(()),
        Err(CompatibilityError::Incompatible) if allow_mix_temp => return Ok(()),
        Err(CompatibilityError::Incompatible) => INCOMPATIBLE,
        Err(CompatibilityError::InvalidRange) => INVALID_RANGE,
    };
    Err(SliceError::FilamentsDifferentTemperature {
        message: message.to_owned(),
        plate_index: plate,
        layer_height: f64::from(settings.process.object.layer_height.0 as f32),
        wall_loops: settings.process.region.wall_loops.0,
        sparse_infill_density: f64::from(settings.process.region.sparse_infill_density.0 as f32),
    })
}

enum CompatibilityError {
    Incompatible,
    InvalidRange,
}

fn check(settings: &ProjectSettings, used: &[usize]) -> Result<(), CompatibilityError> {
    if used.len() < 2 {
        return Ok(());
    }
    let print = &settings.filament.print;
    let mut resolved = Vec::with_capacity(used.len());
    for &slot in used {
        let mut low = get_at(&print.nozzle_temperature_range_low.0, slot).0;
        let mut high = get_at(&print.nozzle_temperature_range_high.0, slot).0;
        if low == 0 || high == 0 {
            let (default_low, default_high) =
                material::range(get_at(&settings.filament.gcode.filament_type.0, slot));
            if low == 0 {
                low = default_low;
            }
            if high == 0 {
                high = default_high;
            }
        }
        if low >= high {
            return Err(CompatibilityError::InvalidRange);
        }
        let temperature = get_at(&print.nozzle_temperature.0, slot).0;
        resolved.push((temperature, low..=high));
    }
    for (index, (temperature, range)) in resolved.iter().enumerate() {
        if resolved[index + 1..]
            .iter()
            .any(|(other_temperature, other_range)| {
                !range.contains(other_temperature) || !other_range.contains(temperature)
            })
        {
            return Err(CompatibilityError::Incompatible);
        }
    }
    Ok(())
}

fn get_at<T>(values: &[T], slot: usize) -> &T {
    values.get(slot).unwrap_or(&values[0])
}
