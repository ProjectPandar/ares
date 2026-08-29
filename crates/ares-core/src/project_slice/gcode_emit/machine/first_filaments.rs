use crate::{
    OrcaInts,
    project_slice::{extruders, perimeters::classic::traversal::PreparedPostClassicTraversal},
};

use super::super::value;

#[cfg(test)]
mod tests;

/// `ToolOrdering::cal_non_support_filaments` and the physical-hotend remap in
/// `GCode.cpp:2721-2851`. Unused physical hotends remain `-1`.
pub(super) fn resolve(traversal: &PreparedPostClassicTraversal) -> (value::Value, value::Value) {
    let settings = &traversal.resolved.views.full;
    let physical_count = settings.project.print.nozzle_diameter.0.len().max(1);
    let mut first = vec![-1; physical_count];
    let mut first_non_support = vec![-1; physical_count];
    let filament_map = &settings.project.gcode.filament_map;

    let mut used = extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    used.sort_unstable();
    used.dedup();
    for filament in used {
        let Some(slot) = int_at(filament_map, filament)
            .checked_sub(1)
            .and_then(|slot| usize::try_from(slot).ok())
            .filter(|slot| *slot < physical_count)
        else {
            continue;
        };
        if first[slot] == -1 {
            first[slot] = filament as i32;
        }
        let support = settings
            .filament
            .gcode
            .filament_is_support
            .0
            .get(filament)
            .or_else(|| settings.filament.gcode.filament_is_support.0.first())
            .is_some_and(|value| value.0);
        if !support && first_non_support[slot] == -1 {
            first_non_support[slot] = filament as i32;
        }
    }

    let heterogeneous = settings
        .printer
        .gcode
        .extruder_type
        .0
        .windows(2)
        .any(|pair| pair[0] != pair[1]);
    first = apply_physical_map(
        first,
        &settings.printer.gcode.physical_extruder_map,
        heterogeneous,
    );
    first_non_support = apply_physical_map(
        first_non_support,
        &settings.printer.gcode.physical_extruder_map,
        heterogeneous,
    );

    (as_value(first), as_value(first_non_support))
}

fn int_at(values: &OrcaInts, index: usize) -> i32 {
    values
        .0
        .get(index)
        .or_else(|| values.0.first())
        .map_or(0, |value| value.0)
}

fn apply_physical_map(
    filaments: Vec<i32>,
    physical_map: &OrcaInts,
    heterogeneous: bool,
) -> Vec<i32> {
    if !heterogeneous {
        return filaments;
    }
    let mut remapped = vec![-1; filaments.len()];
    for (index, filament) in filaments.into_iter().enumerate() {
        let target = int_at(physical_map, index);
        if let Ok(target) = usize::try_from(target)
            && target < remapped.len()
        {
            remapped[target] = filament;
        }
    }
    remapped
}

fn as_value(filaments: Vec<i32>) -> value::Value {
    value::Value::List(
        filaments
            .into_iter()
            .map(|filament| value::Value::number(f64::from(filament)))
            .collect(),
    )
}
