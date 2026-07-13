use serde::ser::SerializeMap;

use super::super::FilamentOptions;

pub(super) fn serialize_entries<M>(map: &mut M, value: &FilamentOptions) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "filament_tower_ironing_area",
        &value.gcode.filament_tower_ironing_area,
    )?;
    map.serialize_entry("filament_type", &value.gcode.filament_type)?;
    map.serialize_entry(
        "filament_unloading_speed",
        &value.gcode.filament_unloading_speed,
    )?;
    map.serialize_entry(
        "filament_unloading_speed_start",
        &value.gcode.filament_unloading_speed_start,
    )?;
    map.serialize_entry("filament_vendor", &value.gcode.filament_vendor)?;
    map.serialize_entry("filament_wipe", &value.retract_overrides.filament_wipe)?;
    map.serialize_entry(
        "filament_wipe_distance",
        &value.retract_overrides.filament_wipe_distance,
    )?;
    map.serialize_entry("filament_z_hop", &value.retract_overrides.filament_z_hop)?;
    map.serialize_entry(
        "filament_z_hop_types",
        &value.retract_overrides.filament_z_hop_types,
    )?;
    map.serialize_entry(
        "first_x_layer_fan_speed",
        &value.print.first_x_layer_fan_speed,
    )?;
    map.serialize_entry("full_fan_speed_layer", &value.print.full_fan_speed_layer)?;
    map.serialize_entry("hot_plate_temp", &value.print.hot_plate_temp)?;
    map.serialize_entry(
        "hot_plate_temp_initial_layer",
        &value.print.hot_plate_temp_initial_layer,
    )?;
    map.serialize_entry("idle_temperature", &value.print.idle_temperature)?;
    map.serialize_entry(
        "internal_bridge_fan_speed",
        &value.print.internal_bridge_fan_speed,
    )?;
    map.serialize_entry("ironing_fan_speed", &value.print.ironing_fan_speed)?;
    map.serialize_entry(
        "long_retractions_when_ec",
        &value.gcode.long_retractions_when_ec,
    )?;
    map.serialize_entry("nozzle_temperature", &value.print.nozzle_temperature)?;
    map.serialize_entry(
        "nozzle_temperature_initial_layer",
        &value.print.nozzle_temperature_initial_layer,
    )?;
    map.serialize_entry(
        "nozzle_temperature_range_high",
        &value.print.nozzle_temperature_range_high,
    )?;
    map.serialize_entry(
        "nozzle_temperature_range_low",
        &value.print.nozzle_temperature_range_low,
    )?;
    map.serialize_entry("overhang_fan_speed", &value.print.overhang_fan_speed)?;
    map.serialize_entry(
        "overhang_fan_threshold",
        &value.print.overhang_fan_threshold,
    )?;
    map.serialize_entry("pellet_flow_coefficient", &value.pellet_flow_coefficient)?;
    map.serialize_entry("pressure_advance", &value.gcode.pressure_advance)?;
    map.serialize_entry(
        "reduce_fan_stop_start_freq",
        &value.print.reduce_fan_stop_start_freq,
    )?;
    map.serialize_entry("required_nozzle_HRC", &value.gcode.required_nozzle_hrc)?;
    map.serialize_entry(
        "retraction_distances_when_ec",
        &value.gcode.retraction_distances_when_ec,
    )?;
    map.serialize_entry(
        "slow_down_for_layer_cooling",
        &value.print.slow_down_for_layer_cooling,
    )?;
    map.serialize_entry("slow_down_layer_time", &value.print.slow_down_layer_time)?;
    map.serialize_entry("slow_down_min_speed", &value.print.slow_down_min_speed)?;
    map.serialize_entry("supertack_plate_temp", &value.print.supertack_plate_temp)?;
    map.serialize_entry(
        "supertack_plate_temp_initial_layer",
        &value.print.supertack_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "support_material_interface_fan_speed",
        &value.print.support_material_interface_fan_speed,
    )?;
    map.serialize_entry(
        "temperature_vitrification",
        &value.gcode.temperature_vitrification,
    )?;
    map.serialize_entry(
        "textured_cool_plate_temp",
        &value.print.textured_cool_plate_temp,
    )?;
    map.serialize_entry(
        "textured_cool_plate_temp_initial_layer",
        &value.print.textured_cool_plate_temp_initial_layer,
    )?;
    map.serialize_entry("textured_plate_temp", &value.print.textured_plate_temp)?;
    map.serialize_entry(
        "textured_plate_temp_initial_layer",
        &value.print.textured_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "volumetric_speed_coefficients",
        &value.gcode.volumetric_speed_coefficients,
    )?;
    Ok(())
}
