use serde::ser::SerializeMap;

use super::super::PrinterRemainingOptions;

pub(super) fn serialize_entries<M: SerializeMap>(
    map: &mut M,
    value: &PrinterRemainingOptions,
) -> Result<(), M::Error> {
    map.serialize_entry("gcode_skip_config_block", &value.gcode_skip_config_block)?;
    map.serialize_entry("build_plate_tilt_x", &value.build_plate_tilt_x)?;
    map.serialize_entry("build_plate_tilt_y", &value.build_plate_tilt_y)?;
    map.serialize_entry("belt_printer", &value.belt_printer)?;
    map.serialize_entry("belt_printer_infinite_y", &value.belt_printer_infinite_y)?;
    map.serialize_entry(
        "belt_slice_rotation_angle",
        &value.belt_slice_rotation_angle,
    )?;
    map.serialize_entry("belt_slice_rotation", &value.belt_slice_rotation)?;
    map.serialize_entry(
        "belt_support_floor_offset",
        &value.belt_support_floor_offset,
    )?;
    map.serialize_entry("gcode_remap_x", &value.gcode_remap_x)?;
    map.serialize_entry("gcode_remap_y", &value.gcode_remap_y)?;
    map.serialize_entry("gcode_remap_z", &value.gcode_remap_z)?;
    map.serialize_entry("belt_frame_tilt_decouple", &value.belt_frame_tilt_decouple)?;
    map.serialize_entry("belt_frame_tilt_angle", &value.belt_frame_tilt_angle)?;
    map.serialize_entry(
        "printer_plugin_config_overrides",
        &value.printer_plugin_config_overrides,
    )?;
    map.serialize_entry(
        "extruder_clearance_dist_to_rod",
        &value.extruder_clearance_dist_to_rod,
    )?;
    map.serialize_entry("fan_direction", &value.fan_direction)?;
    map.serialize_entry("cooling_filter_enabled", &value.cooling_filter_enabled)?;
    map.serialize_entry(
        "wait_for_temp_on_wipe_tower",
        &value.wait_for_temp_on_wipe_tower,
    )?;
    map.serialize_entry("enable_belt_purge_tower", &value.enable_belt_purge_tower)?;
    map.serialize_entry(
        "imex_firmware_managed_zones",
        &value.imex_firmware_managed_zones,
    )?;
    map.serialize_entry("imex_viz_theme", &value.imex_viz_theme)?;
    Ok(())
}
