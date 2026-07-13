use serde::ser::SerializeMap;

use super::super::FilamentOptions;

pub(super) fn serialize_entries<M>(map: &mut M, value: &FilamentOptions) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "filament_extruder_variant",
        &value.gcode.filament_extruder_variant,
    )?;
    map.serialize_entry("filament_flow_ratio", &value.gcode.filament_flow_ratio)?;
    map.serialize_entry("filament_flush_temp", &value.gcode.filament_flush_temp)?;
    map.serialize_entry(
        "filament_flush_volumetric_speed",
        &value.gcode.filament_flush_volumetric_speed,
    )?;
    map.serialize_entry("filament_ironing_flow", &value.region.filament_ironing_flow)?;
    map.serialize_entry(
        "filament_ironing_inset",
        &value.region.filament_ironing_inset,
    )?;
    map.serialize_entry(
        "filament_ironing_spacing",
        &value.region.filament_ironing_spacing,
    )?;
    map.serialize_entry(
        "filament_ironing_speed",
        &value.region.filament_ironing_speed,
    )?;
    map.serialize_entry("filament_is_support", &value.gcode.filament_is_support)?;
    map.serialize_entry(
        "filament_loading_speed",
        &value.gcode.filament_loading_speed,
    )?;
    map.serialize_entry(
        "filament_loading_speed_start",
        &value.gcode.filament_loading_speed_start,
    )?;
    map.serialize_entry(
        "filament_long_retractions_when_cut",
        &value.retract_overrides.filament_long_retractions_when_cut,
    )?;
    map.serialize_entry(
        "filament_max_volumetric_speed",
        &value.gcode.filament_max_volumetric_speed,
    )?;
    map.serialize_entry(
        "filament_minimal_purge_on_wipe_tower",
        &value.gcode.filament_minimal_purge_on_wipe_tower,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming",
        &value.gcode.filament_multitool_ramming,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming_flow",
        &value.gcode.filament_multitool_ramming_flow,
    )?;
    map.serialize_entry(
        "filament_multitool_ramming_volume",
        &value.gcode.filament_multitool_ramming_volume,
    )?;
    map.serialize_entry("filament_notes", &value.print.filament_notes)?;
    map.serialize_entry("filament_printable", &value.gcode.filament_printable)?;
    map.serialize_entry(
        "filament_ramming_parameters",
        &value.gcode.filament_ramming_parameters,
    )?;
    map.serialize_entry(
        "filament_retract_before_wipe",
        &value.retract_overrides.filament_retract_before_wipe,
    )?;
    map.serialize_entry(
        "filament_retract_lift_above",
        &value.retract_overrides.filament_retract_lift_above,
    )?;
    map.serialize_entry(
        "filament_retract_lift_below",
        &value.retract_overrides.filament_retract_lift_below,
    )?;
    map.serialize_entry(
        "filament_retract_lift_enforce",
        &value.retract_overrides.filament_retract_lift_enforce,
    )?;
    map.serialize_entry(
        "filament_retract_restart_extra",
        &value.retract_overrides.filament_retract_restart_extra,
    )?;
    map.serialize_entry(
        "filament_retract_when_changing_layer",
        &value.retract_overrides.filament_retract_when_changing_layer,
    )?;
    map.serialize_entry(
        "filament_retraction_distances_when_cut",
        &value
            .retract_overrides
            .filament_retraction_distances_when_cut,
    )?;
    map.serialize_entry(
        "filament_retraction_length",
        &value.retract_overrides.filament_retraction_length,
    )?;
    map.serialize_entry(
        "filament_retraction_minimum_travel",
        &value.retract_overrides.filament_retraction_minimum_travel,
    )?;
    map.serialize_entry(
        "filament_retraction_speed",
        &value.retract_overrides.filament_retraction_speed,
    )?;
    map.serialize_entry("filament_shrink", &value.print.filament_shrink)?;
    map.serialize_entry(
        "filament_shrinkage_compensation_z",
        &value.print.filament_shrinkage_compensation_z,
    )?;
    map.serialize_entry("filament_soluble", &value.gcode.filament_soluble)?;
    map.serialize_entry(
        "filament_stamping_distance",
        &value.gcode.filament_stamping_distance,
    )?;
    map.serialize_entry(
        "filament_stamping_loading_speed",
        &value.gcode.filament_stamping_loading_speed,
    )?;
    map.serialize_entry("filament_start_gcode", &value.gcode.filament_start_gcode)?;
    map.serialize_entry(
        "filament_toolchange_delay",
        &value.gcode.filament_toolchange_delay,
    )?;
    map.serialize_entry(
        "filament_tower_interface_pre_extrusion_dist",
        &value.gcode.filament_tower_interface_pre_extrusion_dist,
    )?;
    map.serialize_entry(
        "filament_tower_interface_pre_extrusion_length",
        &value.gcode.filament_tower_interface_pre_extrusion_length,
    )?;
    map.serialize_entry(
        "filament_tower_interface_print_temp",
        &value.gcode.filament_tower_interface_print_temp,
    )?;
    map.serialize_entry(
        "filament_tower_interface_purge_volume",
        &value.gcode.filament_tower_interface_purge_volume,
    )?;
    Ok(())
}
