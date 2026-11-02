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
        &Requests {
            ids: &source.printer.gcode.printer_extruder_id.0,
            variants: &source.printer.gcode.printer_extruder_variant.0,
            groups: &source.printer.remaining.extruder_variant_list.0,
            key: "printer_extruder_variant",
            miss_scans_ids: false,
        },
        physical_requests(&active.0),
    )
}

pub(super) fn resolve_printer_indices_for_variant_two(
    source: &ProjectSettings,
    active: &ActiveVariants,
) -> Vec<usize> {
    physical_requests(&active.0)
        .map(|(requested_id, requested_variant)| {
            // PrintApply.cpp:1425 selects variant two before variant one reduces
            // the id/variant metadata used by get_index_for_extruder.
            get_index_for_extruder(
                (requested_id, requested_variant),
                Some(&source.printer.gcode.printer_extruder_id.0),
                &source.printer.gcode.printer_extruder_variant.0,
                &source.printer.remaining.extruder_variant_list.0,
                1,
            )
            .unwrap_or(0)
        })
        .collect()
}

pub(super) fn resolve_process_indices(
    source: &ProjectSettings,
    active: &ActiveVariants,
) -> Result<Vec<usize>, SliceError> {
    resolve_requests(
        &Requests {
            ids: &source.process.region.print_extruder_id.0,
            variants: &source.process.region.print_extruder_variant.0,
            groups: &source.printer.remaining.extruder_variant_list.0,
            key: "print_extruder_variant",
            miss_scans_ids: false,
        },
        physical_requests(&active.0),
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
        &Requests {
            ids: &source.project.preset.filament_self_index.0,
            variants,
            groups: &source.printer.remaining.extruder_variant_list.0,
            key: "filament_extruder_variant",
            miss_scans_ids: true,
        },
        requests,
    )
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

/// The variant table one `update_values_to_printer_extruders*` overload
/// resolves against; `miss_scans_ids` selects the overload's miss policy.
struct Requests<'a> {
    ids: &'a [OrcaInt],
    variants: &'a [String],
    groups: &'a [String],
    key: &'a str,
    miss_scans_ids: bool,
}

fn resolve_requests<'a>(
    table: &Requests<'_>,
    requests: impl IntoIterator<Item = (i32, &'a str)>,
) -> Result<Vec<usize>, SliceError> {
    let Requests {
        ids,
        variants,
        groups,
        key,
        miss_scans_ids,
    } = table;
    let variant_key = key;
    let variant_groups = groups;
    if variants.is_empty() {
        return Err(invalid(variant_key));
    }
    let requests = requests.into_iter().collect::<Vec<_>>();
    // ConfigOptionVector::get_at repeats its first value when a serialized
    // vector carries one shared value (`Config.hpp:322-330`). Orca exports
    // this compact form for multi-tool printers whose variants are equal.
    let shared_variant = &variants[0];
    if variants.iter().all(|variant| variant == shared_variant)
        && requests
            .iter()
            .all(|(_, requested_variant)| *requested_variant == shared_variant)
        && ids
            .first()
            .is_none_or(|first| ids.iter().all(|id| id == first))
    {
        return Ok(vec![0; requests.len()]);
    }

    let indices = requests
        .into_iter()
        .map(|(requested_id, requested_variant)| {
            // `update_values_to_printer_extruders` falls back to variant
            // index 0 silently on a miss — "expected during transient UI
            // states" (PrintConfig.cpp:9677-9681, the extruder loop
            // branch the CLI slicing path takes). A miss is not an error.
            // The multiple-filaments overload additionally rescans the
            // id vector for this entry's own id and prefers that
            // position (PrintConfig.cpp:9849-9863).
            get_index_for_extruder(
                (requested_id, requested_variant),
                Some(ids),
                variants,
                variant_groups,
                1,
            )
            .unwrap_or_else(|| {
                if *miss_scans_ids {
                    ids.iter().position(|id| id.0 == requested_id).unwrap_or(0)
                } else {
                    0
                }
            })
        })
        .collect::<Vec<_>>();
    select_stride(variants, &indices, 1, variant_key)?;
    Ok(indices)
}

// PrintConfig.cpp:9004–9060. None is upstream -1; callers own miss policy.
pub(crate) fn get_index_for_extruder(
    (requested_id, requested_variant): (i32, &str),
    ids: Option<&[OrcaInt]>,
    variants: &[String],
    variant_groups: &[String],
    stride: usize,
) -> Option<usize> {
    let complete_ids = ids.is_some_and(|ids| ids.len() >= variants.len());
    let generated = (!complete_ids).then(|| generated_ids(variant_groups));
    variants.iter().enumerate().find_map(|(index, variant)| {
        if variant != requested_variant {
            return None;
        }
        let matches = ids.is_none_or(|ids| {
            let id = if complete_ids {
                ids[index].0
            } else {
                // Missing groups or exhausted flattened indices return0
                // in the source generator (PrintConfig.cpp:9012–9013,9028).
                generated
                    .as_ref()
                    .and_then(|ids| ids.get(index))
                    .copied()
                    .unwrap_or(0)
            };
            id == requested_id
        });
        matches.then_some(index * stride)
    })
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

pub(crate) fn canonical_variant(extruder: ExtruderType, volume: NozzleVolumeType) -> String {
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
