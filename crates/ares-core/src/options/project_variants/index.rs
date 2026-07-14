use std::collections::BTreeSet;

use crate::{
    SliceError,
    options::{ExtruderType, NozzleVolumeType, OrcaInt, ProjectSettings},
};

use super::select::select_stride;

pub(super) struct ActiveVariants(Vec<String>);

pub(super) fn resolve_activation(
    source: &ProjectSettings,
) -> Result<Option<ActiveVariants>, SliceError> {
    let physical_count = physical_count(source)?;
    let tokens = guard_tokens(source, physical_count)?;
    if physical_count == 1 && tokens.len() <= 1 {
        return Ok(None);
    }

    Ok(Some(ActiveVariants(active_variants(
        source,
        physical_count,
    )?)))
}

pub(super) fn resolve_printer_indices(
    source: &ProjectSettings,
    active: &ActiveVariants,
) -> Result<Vec<usize>, SliceError> {
    resolve_requests(
        &source.printer.gcode.printer_extruder_id.0,
        &source.printer.gcode.printer_extruder_variant.0,
        physical_requests(&active.0),
        &source.printer.remaining.extruder_variant_list.0,
        "printer_extruder_variant",
    )
}

pub(super) fn resolve_process_indices(
    source: &ProjectSettings,
    active: &ActiveVariants,
) -> Result<Vec<usize>, SliceError> {
    resolve_requests(
        &source.process.region.print_extruder_id.0,
        &source.process.region.print_extruder_variant.0,
        physical_requests(&active.0),
        &source.printer.remaining.extruder_variant_list.0,
        "print_extruder_variant",
    )
}

pub(super) fn resolve_filament_indices(
    source: &ProjectSettings,
    filament_map: &crate::options::OrcaInts,
    active: &ActiveVariants,
) -> Result<Vec<usize>, SliceError> {
    let variants = &source.filament.gcode.filament_extruder_variant.0;
    if variants.is_empty() {
        return Err(invalid("filament_extruder_variant"));
    }

    let requests = filament_map
        .0
        .iter()
        .enumerate()
        .map(|(logical_index, OrcaInt(physical_id))| {
            let physical_index = usize::try_from(*physical_id)
                .ok()
                .and_then(|id| id.checked_sub(1))
                .filter(|&index| index < active.0.len())
                .ok_or_else(|| invalid("filament_map"))?;
            Ok((
                (logical_index + 1) as i32,
                active.0[physical_index].as_str(),
            ))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;

    resolve_requests(
        &source.project.preset.filament_self_index.0,
        variants,
        requests,
        &source.printer.remaining.extruder_variant_list.0,
        "filament_extruder_variant",
    )
}

#[cfg(test)]
pub(crate) fn inspect_printer_indices_for_test(
    source: &ProjectSettings,
) -> Result<Option<Vec<usize>>, SliceError> {
    let Some(active) = resolve_activation(source)? else {
        return Ok(None);
    };

    resolve_printer_indices(source, &active).map(Some)
}

fn physical_count(source: &ProjectSettings) -> Result<usize, SliceError> {
    let count = source.project.print.nozzle_diameter.0.len();
    if count == 0 {
        Err(invalid("nozzle_diameter"))
    } else {
        Ok(count)
    }
}

fn guard_tokens(
    source: &ProjectSettings,
    physical_count: usize,
) -> Result<BTreeSet<String>, SliceError> {
    let groups = &source.printer.remaining.extruder_variant_list.0;
    let mut tokens = BTreeSet::new();
    for index in 0..physical_count {
        let group = get_at(groups, index, "extruder_variant_list")?;
        tokens.extend(split_compressed_commas(group).map(str::to_owned));
    }
    Ok(tokens)
}

fn active_variants(
    source: &ProjectSettings,
    physical_count: usize,
) -> Result<Vec<String>, SliceError> {
    (0..physical_count)
        .map(|index| {
            let extruder_type = get_at(
                &source.printer.gcode.extruder_type.0,
                index,
                "extruder_type",
            )?;
            let nozzle_volume_type = get_at(
                &source.project.gcode.nozzle_volume_type.0,
                index,
                "nozzle_volume_type",
            )?;
            Ok(canonical_variant(*extruder_type, *nozzle_volume_type))
        })
        .collect()
}

fn physical_requests(active_variants: &[String]) -> impl Iterator<Item = (i32, &str)> {
    active_variants
        .iter()
        .enumerate()
        .map(|(index, variant)| ((index + 1) as i32, variant.as_str()))
}

fn resolve_requests<'a>(
    ids: &[OrcaInt],
    variants: &[String],
    requests: impl IntoIterator<Item = (i32, &'a str)>,
    variant_groups: &[String],
    variant_key: &str,
) -> Result<Vec<usize>, SliceError> {
    if variants.is_empty() {
        return Err(invalid(variant_key));
    }

    let complete_ids = ids.len() >= variants.len();
    let generated_ids = (!complete_ids).then(|| generated_ids(variant_groups));
    let indices = requests
        .into_iter()
        .map(|(requested_id, requested_variant)| {
            variants
                .iter()
                .enumerate()
                .find_map(|(index, variant)| {
                    if variant != requested_variant {
                        return None;
                    }
                    let id = if complete_ids {
                        ids[index].0
                    } else {
                        generated_ids
                            .as_ref()
                            .and_then(|ids| ids.get(index))
                            .copied()
                            .unwrap_or(0)
                    };
                    (id == requested_id).then_some(index)
                })
                .ok_or_else(|| invalid(variant_key))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    select_stride(variants, &indices, 1, variant_key)?;
    Ok(indices)
}

fn generated_ids(variant_groups: &[String]) -> Vec<i32> {
    variant_groups
        .iter()
        .enumerate()
        .flat_map(|(group_index, group)| {
            split_compressed_commas(group).filter_map(move |token| {
                (!token.trim().is_empty()).then_some((group_index + 1) as i32)
            })
        })
        .collect()
}

fn split_compressed_commas(value: &str) -> impl Iterator<Item = &str> {
    let parts: Vec<_> = value.split(',').collect();
    let last = parts.len() - 1;
    parts
        .into_iter()
        .enumerate()
        .filter_map(move |(index, token)| {
            (!token.is_empty() || index == 0 || index == last).then_some(token)
        })
}

fn canonical_variant(extruder: ExtruderType, volume: NozzleVolumeType) -> String {
    let extruder = match extruder {
        ExtruderType::DirectDrive => "Direct Drive",
        ExtruderType::Bowden => "Bowden",
    };
    let volume = match volume {
        NozzleVolumeType::Standard => "Standard",
        NozzleVolumeType::HighFlow => "High Flow",
    };
    format!("{extruder} {volume}")
}

fn get_at<'a, T>(values: &'a [T], index: usize, key: &str) -> Result<&'a T, SliceError> {
    values
        .get(index)
        .or_else(|| values.first())
        .ok_or_else(|| invalid(key))
}

fn invalid(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}
