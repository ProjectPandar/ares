use super::super::{Nullable, OrcaInt, ProjectSettings};
use crate::SliceError;

const ENABLE_FILAMENT_LONG_RETRACTION: i32 = 2;

macro_rules! overlay {
    ($machine:expr, $filament:expr, $map:expr, $key:literal) => {
        apply_nullable(&mut $machine.0, $filament, $map, $key)?
    };
}

pub(super) fn apply(
    runtime: &mut ProjectSettings,
    full: &ProjectSettings,
) -> Result<(), SliceError> {
    let map = &full.project.gcode.filament_map.0;
    let overrides = &full.filament.retract_overrides;

    overlay!(
        runtime.project.gcode.deretraction_speed,
        &overrides.filament_deretraction_speed,
        map,
        "filament_deretraction_speed"
    );
    if full.printer.gcode.enable_long_retraction_when_cut.0 == ENABLE_FILAMENT_LONG_RETRACTION {
        overlay!(
            runtime.printer.gcode.long_retractions_when_cut,
            &overrides.filament_long_retractions_when_cut,
            map,
            "filament_long_retractions_when_cut"
        );
    } else {
        let nil_overrides = vec![Nullable::Nil; overrides.filament_long_retractions_when_cut.len()];
        overlay!(
            runtime.printer.gcode.long_retractions_when_cut,
            &nil_overrides,
            map,
            "filament_long_retractions_when_cut"
        );
    }
    overlay!(
        runtime.project.gcode.retract_before_wipe,
        &overrides.filament_retract_before_wipe,
        map,
        "filament_retract_before_wipe"
    );
    overlay!(
        runtime.project.gcode.retract_lift_above,
        &overrides.filament_retract_lift_above,
        map,
        "filament_retract_lift_above"
    );
    overlay!(
        runtime.project.gcode.retract_lift_below,
        &overrides.filament_retract_lift_below,
        map,
        "filament_retract_lift_below"
    );
    overlay!(
        runtime.printer.gcode.retract_lift_enforce,
        &overrides.filament_retract_lift_enforce,
        map,
        "filament_retract_lift_enforce"
    );
    overlay!(
        runtime.project.gcode.retract_restart_extra,
        &overrides.filament_retract_restart_extra,
        map,
        "filament_retract_restart_extra"
    );
    overlay!(
        runtime.project.print.retract_when_changing_layer,
        &overrides.filament_retract_when_changing_layer,
        map,
        "filament_retract_when_changing_layer"
    );
    if full.printer.gcode.enable_long_retraction_when_cut.0 == ENABLE_FILAMENT_LONG_RETRACTION {
        overlay!(
            runtime.printer.gcode.retraction_distances_when_cut,
            &overrides.filament_retraction_distances_when_cut,
            map,
            "filament_retraction_distances_when_cut"
        );
    }
    // The fixed source fills the bool temporary instead, leaving its float override empty here.
    overlay!(
        runtime.project.gcode.retraction_length,
        &overrides.filament_retraction_length,
        map,
        "filament_retraction_length"
    );
    overlay!(
        runtime.project.print.retraction_minimum_travel,
        &overrides.filament_retraction_minimum_travel,
        map,
        "filament_retraction_minimum_travel"
    );
    overlay!(
        runtime.project.gcode.retraction_speed,
        &overrides.filament_retraction_speed,
        map,
        "filament_retraction_speed"
    );
    overlay!(
        runtime.project.print.wipe,
        &overrides.filament_wipe,
        map,
        "filament_wipe"
    );
    overlay!(
        runtime.project.print.wipe_distance,
        &overrides.filament_wipe_distance,
        map,
        "filament_wipe_distance"
    );
    overlay!(
        runtime.project.gcode.z_hop,
        &overrides.filament_z_hop,
        map,
        "filament_z_hop"
    );
    overlay!(
        runtime.printer.gcode.z_hop_types,
        &overrides.filament_z_hop_types,
        map,
        "filament_z_hop_types"
    );
    Ok(())
}

fn apply_nullable<T: Clone>(
    machine: &mut Vec<T>,
    filament: &[Nullable<T>],
    filament_map: &[OrcaInt],
    key: &str,
) -> Result<(), SliceError> {
    if machine.is_empty() || filament.is_empty() {
        return Ok(());
    }
    if filament.len() != filament_map.len() {
        return Err(SliceError::InvalidInput(format!(
            "{key} length must match filament_map"
        )));
    }

    let defaults = machine.clone();
    machine.resize(filament.len(), defaults[0].clone());
    for ((target, override_value), OrcaInt(mapped_extruder)) in
        machine.iter_mut().zip(filament).zip(filament_map)
    {
        *target = match override_value {
            Nullable::Value(value) => value.clone(),
            Nullable::Nil => {
                let index = mapped_extruder
                    .checked_sub(1)
                    .and_then(|index| usize::try_from(index).ok())
                    .filter(|&index| index < defaults.len())
                    .unwrap_or(0);
                defaults[index].clone()
            }
        };
    }
    Ok(())
}
