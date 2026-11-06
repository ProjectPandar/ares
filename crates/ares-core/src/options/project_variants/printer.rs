use crate::{SliceError, options::ProjectSettings};

use super::select::select_stride;

macro_rules! select_field {
    ($field:expr, $indices:expr, $key:literal) => {{
        let selected = select_stride(&$field.0, $indices, 1, $key)?;
        $field.0 = selected;
    }};
}

macro_rules! select_stride_two_field {
    ($field:expr, $indices:expr, $key:literal) => {{
        let selected = select_stride(&$field.0, $indices, 2, $key)?;
        $field.0 = selected;
    }};
}

pub(super) fn materialize_variant_one(
    settings: &mut ProjectSettings,
    indices: &[usize],
) -> Result<(), SliceError> {
    select_field!(
        settings.project.gcode.deretraction_speed,
        indices,
        "deretraction_speed"
    );
    select_field!(
        settings.printer.gcode.long_retractions_when_cut,
        indices,
        "long_retractions_when_cut"
    );
    select_field!(
        settings.printer.gcode.nozzle_flush_dataset,
        indices,
        "nozzle_flush_dataset"
    );
    select_field!(
        settings.printer.gcode.hotend_cooling_rate,
        indices,
        "hotend_cooling_rate"
    );
    select_field!(
        settings.printer.gcode.hotend_heating_rate,
        indices,
        "hotend_heating_rate"
    );
    select_field!(settings.printer.gcode.nozzle_type, indices, "nozzle_type");
    select_field!(
        settings.printer.remaining.nozzle_volume,
        indices,
        "nozzle_volume"
    );
    select_field!(
        settings.printer.gcode.printer_extruder_id,
        indices,
        "printer_extruder_id"
    );
    select_field!(
        settings.printer.gcode.printer_extruder_variant,
        indices,
        "printer_extruder_variant"
    );
    select_field!(
        settings.project.gcode.retract_before_wipe,
        indices,
        "retract_before_wipe"
    );
    select_field!(
        settings.project.gcode.retract_length_toolchange,
        indices,
        "retract_length_toolchange"
    );
    select_field!(
        settings.project.gcode.retract_lift_above,
        indices,
        "retract_lift_above"
    );
    select_field!(
        settings.project.gcode.retract_lift_below,
        indices,
        "retract_lift_below"
    );
    select_field!(
        settings.printer.gcode.retract_lift_enforce,
        indices,
        "retract_lift_enforce"
    );
    select_field!(
        settings.project.gcode.retract_restart_extra,
        indices,
        "retract_restart_extra"
    );
    select_field!(
        settings.project.gcode.retract_restart_extra_toolchange,
        indices,
        "retract_restart_extra_toolchange"
    );
    select_field!(
        settings.project.print.retract_when_changing_layer,
        indices,
        "retract_when_changing_layer"
    );
    select_field!(
        settings.printer.gcode.retraction_distances_when_cut,
        indices,
        "retraction_distances_when_cut"
    );
    select_field!(
        settings.project.gcode.retraction_length,
        indices,
        "retraction_length"
    );
    select_field!(
        settings.project.print.retraction_minimum_travel,
        indices,
        "retraction_minimum_travel"
    );
    select_field!(
        settings.project.gcode.retraction_speed,
        indices,
        "retraction_speed"
    );
    select_field!(settings.printer.gcode.travel_slope, indices, "travel_slope");
    select_field!(settings.project.print.wipe, indices, "wipe");
    select_field!(
        settings.project.print.wipe_distance,
        indices,
        "wipe_distance"
    );
    select_field!(settings.project.gcode.z_hop, indices, "z_hop");
    select_field!(settings.printer.gcode.z_hop_types, indices, "z_hop_types");
    Ok(())
}

pub(super) fn materialize_variant_two(
    settings: &mut ProjectSettings,
    indices: &[usize],
) -> Result<(), SliceError> {
    let stride_indices = indices.iter().map(|index| index * 2).collect::<Vec<_>>();
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_e,
        &stride_indices,
        "machine_max_acceleration_e"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_extruding,
        &stride_indices,
        "machine_max_acceleration_extruding"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_retracting,
        &stride_indices,
        "machine_max_acceleration_retracting"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_travel,
        &stride_indices,
        "machine_max_acceleration_travel"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_x,
        &stride_indices,
        "machine_max_acceleration_x"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_y,
        &stride_indices,
        "machine_max_acceleration_y"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_acceleration_z,
        &stride_indices,
        "machine_max_acceleration_z"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_jerk_e,
        &stride_indices,
        "machine_max_jerk_e"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_jerk_x,
        &stride_indices,
        "machine_max_jerk_x"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_jerk_y,
        &stride_indices,
        "machine_max_jerk_y"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_jerk_z,
        &stride_indices,
        "machine_max_jerk_z"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_junction_deviation,
        &stride_indices,
        "machine_max_junction_deviation"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_speed_e,
        &stride_indices,
        "machine_max_speed_e"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_speed_x,
        &stride_indices,
        "machine_max_speed_x"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_speed_y,
        &stride_indices,
        "machine_max_speed_y"
    );
    select_stride_two_field!(
        settings.printer.machine.machine_max_speed_z,
        &stride_indices,
        "machine_max_speed_z"
    );
    Ok(())
}

pub(super) fn materialize_process(
    settings: &mut ProjectSettings,
    indices: &[usize],
) -> Result<(), SliceError> {
    select_field!(
        settings.process.object.default_acceleration,
        indices,
        "default_acceleration"
    );
    select_field!(
        settings.process.object.outer_wall_acceleration,
        indices,
        "outer_wall_acceleration"
    );
    select_field!(
        settings.process.object.inner_wall_acceleration,
        indices,
        "inner_wall_acceleration"
    );
    select_field!(
        settings.process.object.top_surface_acceleration,
        indices,
        "top_surface_acceleration"
    );
    select_field!(
        settings.process.object.initial_layer_acceleration,
        indices,
        "initial_layer_acceleration"
    );
    select_field!(
        settings.process.object.bridge_acceleration,
        indices,
        "bridge_acceleration"
    );
    select_field!(
        settings.process.object.travel_acceleration,
        indices,
        "travel_acceleration"
    );
    select_field!(
        settings.process.object.sparse_infill_acceleration,
        indices,
        "sparse_infill_acceleration"
    );
    select_field!(
        settings.process.object.internal_solid_infill_acceleration,
        indices,
        "internal_solid_infill_acceleration"
    );
    select_field!(
        settings.process.object.default_jerk,
        indices,
        "default_jerk"
    );
    select_field!(
        settings.process.object.outer_wall_jerk,
        indices,
        "outer_wall_jerk"
    );
    select_field!(
        settings.process.object.inner_wall_jerk,
        indices,
        "inner_wall_jerk"
    );
    select_field!(settings.process.object.infill_jerk, indices, "infill_jerk");
    select_field!(
        settings.process.object.top_surface_jerk,
        indices,
        "top_surface_jerk"
    );
    select_field!(
        settings.process.object.initial_layer_jerk,
        indices,
        "initial_layer_jerk"
    );
    select_field!(settings.process.object.travel_jerk, indices, "travel_jerk");
    select_field!(
        settings.process.gcode.initial_layer_travel_acceleration,
        indices,
        "initial_layer_travel_acceleration"
    );
    settings.process.region.inner_wall_speed.0 = select_stride(
        &settings.process.region.inner_wall_speed.0,
        indices,
        1,
        "inner_wall_speed",
    )?;
    settings.process.region.outer_wall_speed.0 = select_stride(
        &settings.process.region.outer_wall_speed.0,
        indices,
        1,
        "outer_wall_speed",
    )?;
    settings.process.region.enable_overhang_speed.0 = select_stride(
        &settings.process.region.enable_overhang_speed.0,
        indices,
        1,
        "enable_overhang_speed",
    )?;
    settings.process.region.slowdown_for_curled_perimeters.0 = select_stride(
        &settings.process.region.slowdown_for_curled_perimeters.0,
        indices,
        1,
        "slowdown_for_curled_perimeters",
    )?;
    select_field!(
        settings.process.region.print_extruder_id,
        indices,
        "print_extruder_id"
    );
    select_field!(
        settings.process.region.print_extruder_variant,
        indices,
        "print_extruder_variant"
    );
    Ok(())
}
