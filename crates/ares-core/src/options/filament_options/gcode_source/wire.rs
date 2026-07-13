use serde::{Serialize, Serializer, ser::SerializeMap};

use super::FilamentGCodeSourceOptions;

impl Serialize for FilamentGCodeSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(53))?;
        serialize_entries(&mut map, self)?;
        map.end()
    }
}

pub(crate) fn serialize_entries<M>(
    map: &mut M,
    value: &FilamentGCodeSourceOptions,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "adaptive_pressure_advance",
        &value.adaptive_pressure_advance,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_bridges",
        &value.adaptive_pressure_advance_bridges,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_model",
        &value.adaptive_pressure_advance_model,
    )?;
    map.serialize_entry(
        "adaptive_pressure_advance_overhangs",
        &value.adaptive_pressure_advance_overhangs,
    )?;
    map.serialize_entry("default_filament_colour", &value.default_filament_colour)?;
    map.serialize_entry("enable_pressure_advance", &value.enable_pressure_advance)?;
    map.serialize_entry(
        "filament_adaptive_volumetric_speed",
        &value.filament_adaptive_volumetric_speed,
    )?;
    map.serialize_entry(
        "filament_adhesiveness_category",
        &value.filament_adhesiveness_category,
    )?;
    map.serialize_entry(
        "filament_change_extrusion_role_gcode",
        &value.filament_change_extrusion_role_gcode,
    )?;
    map.serialize_entry("filament_change_length", &value.filament_change_length)?;
    map.serialize_entry("filament_colour", &value.filament_colour)?;
    map.serialize_entry(
        "filament_cooling_before_tower",
        &value.filament_cooling_before_tower,
    )?;
    map.serialize_entry(
        "filament_cooling_final_speed",
        &value.filament_cooling_final_speed,
    )?;
    map.serialize_entry(
        "filament_cooling_initial_speed",
        &value.filament_cooling_initial_speed,
    )?;
    map.serialize_entry("filament_cooling_moves", &value.filament_cooling_moves)?;
    map.serialize_entry("filament_cost", &value.filament_cost)?;
    map.serialize_entry("filament_density", &value.filament_density)?;
    map.serialize_entry("filament_diameter", &value.filament_diameter)?;
    map.serialize_entry("filament_end_gcode", &value.filament_end_gcode)?;
    map.serialize_entry(
        "filament_extruder_variant",
        &value.filament_extruder_variant,
    )?;
    map.serialize_entry("filament_flow_ratio", &value.filament_flow_ratio)?;
    map.serialize_entry("filament_flush_temp", &value.filament_flush_temp)?;
    map.serialize_entry(
        "filament_flush_volumetric_speed",
        &value.filament_flush_volumetric_speed,
    )?;
    map.serialize_entry("filament_is_support", &value.filament_is_support)?;
    map.serialize_entry("filament_loading_speed", &value.filament_loading_speed)?;
    map.serialize_entry(
        "filament_loading_speed_start",
        &value.filament_loading_speed_start,
    )?;
    map.serialize_entry(
        "filament_max_volumetric_speed",
        &value.filament_max_volumetric_speed,
    )?;
    map.serialize_entry(
        "filament_minimal_purge_on_wipe_tower",
        &value.filament_minimal_purge_on_wipe_tower,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming",
        &value.filament_multitool_ramming,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming_flow",
        &value.filament_multitool_ramming_flow,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming_volume",
        &value.filament_multitool_ramming_volume,
    )?;
    map.serialize_entry("filament_printable", &value.filament_printable)?;
    map.serialize_entry(
        "filament_ramming_parameters",
        &value.filament_ramming_parameters,
    )?;
    map.serialize_entry("filament_soluble", &value.filament_soluble)?;
    map.serialize_entry(
        "filament_stamping_distance",
        &value.filament_stamping_distance,
    )?;
    map.serialize_entry(
        "filament_stamping_loading_speed",
        &value.filament_stamping_loading_speed,
    )?;
    map.serialize_entry("filament_start_gcode", &value.filament_start_gcode)?;
    map.serialize_entry(
        "filament_toolchange_delay",
        &value.filament_toolchange_delay,
    )?;
    map.serialize_entry(
        "filament_tower_interface_pre_extrusion_dist",
        &value.filament_tower_interface_pre_extrusion_dist,
    )?;
    map.serialize_entry(
        "filament_tower_interface_pre_extrusion_length",
        &value.filament_tower_interface_pre_extrusion_length,
    )?;
    map.serialize_entry(
        "filament_tower_interface_print_temp",
        &value.filament_tower_interface_print_temp,
    )?;
    map.serialize_entry(
        "filament_tower_interface_purge_volume",
        &value.filament_tower_interface_purge_volume,
    )?;
    map.serialize_entry(
        "filament_tower_ironing_area",
        &value.filament_tower_ironing_area,
    )?;
    map.serialize_entry("filament_type", &value.filament_type)?;
    map.serialize_entry("filament_unloading_speed", &value.filament_unloading_speed)?;
    map.serialize_entry(
        "filament_unloading_speed_start",
        &value.filament_unloading_speed_start,
    )?;
    map.serialize_entry("filament_vendor", &value.filament_vendor)?;
    map.serialize_entry("long_retractions_when_ec", &value.long_retractions_when_ec)?;
    map.serialize_entry("pressure_advance", &value.pressure_advance)?;
    map.serialize_entry("required_nozzle_HRC", &value.required_nozzle_hrc)?;
    map.serialize_entry(
        "retraction_distances_when_ec",
        &value.retraction_distances_when_ec,
    )?;
    map.serialize_entry(
        "temperature_vitrification",
        &value.temperature_vitrification,
    )?;
    map.serialize_entry(
        "volumetric_speed_coefficients",
        &value.volumetric_speed_coefficients,
    )?;
    Ok(())
}
