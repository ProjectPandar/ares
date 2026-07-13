use serde::ser::SerializeMap;

use super::super::FilamentOptions;

pub(super) fn serialize_entries<M>(map: &mut M, value: &FilamentOptions) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "activate_air_filtration",
        &value.print.activate_air_filtration,
    )?;
    map.serialize_entry(
        "activate_air_filtration_during_print",
        &value.print.activate_air_filtration_during_print,
    )?;
    map.serialize_entry(
        "activate_air_filtration_on_completion",
        &value.print.activate_air_filtration_on_completion,
    )?;
    map.serialize_entry(
        "activate_chamber_temp_control",
        &value.print.activate_chamber_temp_control,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance",
        &value.gcode.adaptive_pressure_advance,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_bridges",
        &value.gcode.adaptive_pressure_advance_bridges,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_model",
        &value.gcode.adaptive_pressure_advance_model,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_overhangs",
        &value.gcode.adaptive_pressure_advance_overhangs,
    )?;
    map.serialize_entry(
        "additional_cooling_fan_speed",
        &value.print.additional_cooling_fan_speed,
    )?;
    map.serialize_entry(
        "additional_fan_full_speed_layer",
        &value.print.additional_fan_full_speed_layer,
    )?;
    map.serialize_entry(
        "chamber_minimal_temperature",
        &value.print.chamber_minimal_temperature,
    )?;
    map.serialize_entry("chamber_temperature", &value.print.chamber_temperature)?;
    map.serialize_entry(
        "close_additional_fan_first_x_layers",
        &value.print.close_additional_fan_first_x_layers,
    )?;
    map.serialize_entry(
        "close_fan_the_first_x_layers",
        &value.print.close_fan_the_first_x_layers,
    )?;
    map.serialize_entry(
        "complete_print_exhaust_fan_speed",
        &value.print.complete_print_exhaust_fan_speed,
    )?;
    map.serialize_entry("cool_plate_temp", &value.print.cool_plate_temp)?;
    map.serialize_entry(
        "cool_plate_temp_initial_layer",
        &value.print.cool_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "default_filament_colour",
        &value.gcode.default_filament_colour,
    )?;
    map.serialize_entry(
        "dont_slow_down_outer_wall",
        &value.print.dont_slow_down_outer_wall,
    )?;
    map.serialize_entry(
        "during_print_exhaust_fan_speed",
        &value.print.during_print_exhaust_fan_speed,
    )?;
    map.serialize_entry(
        "enable_overhang_bridge_fan",
        &value.print.enable_overhang_bridge_fan,
    )?;
    map.serialize_entry(
        "enable_pressure_advance",
        &value.gcode.enable_pressure_advance,
    )?;
    map.serialize_entry("eng_plate_temp", &value.print.eng_plate_temp)?;
    map.serialize_entry(
        "eng_plate_temp_initial_layer",
        &value.print.eng_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "fan_cooling_layer_time",
        &value.print.fan_cooling_layer_time,
    )?;
    map.serialize_entry("fan_max_speed", &value.print.fan_max_speed)?;
    map.serialize_entry("fan_min_speed", &value.print.fan_min_speed)?;
    map.serialize_entry(
        "filament_adaptive_volumetric_speed",
        &value.gcode.filament_adaptive_volumetric_speed,
    )?;
    map.serialize_entry(
        "filament_adhesiveness_category",
        &value.gcode.filament_adhesiveness_category,
    )?;
    map.serialize_entry(
        "filament_change_extrusion_role_gcode",
        &value.gcode.filament_change_extrusion_role_gcode,
    )?;
    map.serialize_entry(
        "filament_change_length",
        &value.gcode.filament_change_length,
    )?;
    map.serialize_entry("filament_colour", &value.gcode.filament_colour)?;
    map.serialize_entry(
        "filament_cooling_before_tower",
        &value.gcode.filament_cooling_before_tower,
    )?;
    map.serialize_entry(
        "filament_cooling_final_speed",
        &value.gcode.filament_cooling_final_speed,
    )?;
    map.serialize_entry(
        "filament_cooling_initial_speed",
        &value.gcode.filament_cooling_initial_speed,
    )?;
    map.serialize_entry(
        "filament_cooling_moves",
        &value.gcode.filament_cooling_moves,
    )?;
    map.serialize_entry("filament_cost", &value.gcode.filament_cost)?;
    map.serialize_entry("filament_density", &value.gcode.filament_density)?;
    map.serialize_entry(
        "filament_deretraction_speed",
        &value.retract_overrides.filament_deretraction_speed,
    )?;
    map.serialize_entry("filament_diameter", &value.gcode.filament_diameter)?;
    map.serialize_entry("filament_end_gcode", &value.gcode.filament_end_gcode)?;
    Ok(())
}
