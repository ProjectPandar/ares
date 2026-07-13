use serde::ser::SerializeMap;

use super::super::ProcessOptions;

pub(super) fn serialize_entries<M>(map: &mut M, process: &ProcessOptions) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    let ProcessOptions {
        gcode,
        object,
        print,
        region,
        ..
    } = process;
    map.serialize_entry("accel_to_decel_enable", &gcode.accel_to_decel_enable)?;
    map.serialize_entry("accel_to_decel_factor", &gcode.accel_to_decel_factor)?;
    map.serialize_entry(
        "align_infill_direction_to_model",
        &region.align_infill_direction_to_model,
    )?;
    map.serialize_entry("alternate_extra_wall", &region.alternate_extra_wall)?;
    map.serialize_entry("bottom_shell_layers", &region.bottom_shell_layers)?;
    map.serialize_entry("bottom_shell_thickness", &region.bottom_shell_thickness)?;
    map.serialize_entry(
        "bottom_solid_infill_flow_ratio",
        &region.bottom_solid_infill_flow_ratio,
    )?;
    map.serialize_entry("bottom_surface_density", &region.bottom_surface_density)?;
    map.serialize_entry(
        "bottom_surface_filament_id",
        &region.bottom_surface_filament_id,
    )?;
    map.serialize_entry("bottom_surface_pattern", &region.bottom_surface_pattern)?;
    map.serialize_entry("bridge_acceleration", &object.bridge_acceleration)?;
    map.serialize_entry("bridge_angle", &region.bridge_angle)?;
    map.serialize_entry("bridge_density", &region.bridge_density)?;
    map.serialize_entry("bridge_flow", &region.bridge_flow)?;
    map.serialize_entry("bridge_line_width", &region.bridge_line_width)?;
    map.serialize_entry("bridge_no_support", &object.bridge_no_support)?;
    map.serialize_entry("bridge_speed", &region.bridge_speed)?;
    map.serialize_entry(
        "brim_ears_detection_length",
        &object.brim_ears_detection_length,
    )?;
    map.serialize_entry("brim_ears_max_angle", &object.brim_ears_max_angle)?;
    map.serialize_entry("brim_flow_ratio", &object.brim_flow_ratio)?;
    map.serialize_entry("brim_object_gap", &object.brim_object_gap)?;
    map.serialize_entry("brim_type", &object.brim_type)?;
    map.serialize_entry("brim_use_efc_outline", &object.brim_use_efc_outline)?;
    map.serialize_entry("brim_width", &object.brim_width)?;
    map.serialize_entry(
        "calib_flowrate_topinfill_special_order",
        &object.calib_flowrate_topinfill_special_order,
    )?;
    map.serialize_entry("combine_brims", &print.combine_brims)?;
    map.serialize_entry(
        "counterbore_hole_bridging",
        &region.counterbore_hole_bridging,
    )?;
    map.serialize_entry("default_acceleration", &object.default_acceleration)?;
    map.serialize_entry("default_jerk", &object.default_jerk)?;
    map.serialize_entry(
        "default_junction_deviation",
        &object.default_junction_deviation,
    )?;
    map.serialize_entry(
        "detect_narrow_internal_solid_infill",
        &object.detect_narrow_internal_solid_infill,
    )?;
    map.serialize_entry("detect_overhang_wall", &region.detect_overhang_wall)?;
    map.serialize_entry("detect_thin_wall", &region.detect_thin_wall)?;
    map.serialize_entry(
        "dont_filter_internal_bridges",
        &object.dont_filter_internal_bridges,
    )?;
    map.serialize_entry("draft_shield", &print.draft_shield)?;
    map.serialize_entry(
        "elefant_foot_compensation",
        &object.elefant_foot_compensation,
    )?;
    map.serialize_entry(
        "elefant_foot_compensation_layers",
        &object.elefant_foot_compensation_layers,
    )?;
    map.serialize_entry(
        "elefant_foot_layers_density",
        &object.elefant_foot_layers_density,
    )?;
    map.serialize_entry("enable_arc_fitting", &gcode.enable_arc_fitting)?;
    map.serialize_entry(
        "enable_extra_bridge_layer",
        &object.enable_extra_bridge_layer,
    )?;
    map.serialize_entry("enable_overhang_speed", &region.enable_overhang_speed)?;
    map.serialize_entry("enable_prime_tower", &print.enable_prime_tower)?;
    map.serialize_entry("enable_support", &object.enable_support)?;
    map.serialize_entry(
        "enable_tower_interface_cooldown_during_tower",
        &print.enable_tower_interface_cooldown_during_tower,
    )?;
    map.serialize_entry(
        "enable_tower_interface_features",
        &print.enable_tower_interface_features,
    )?;
    map.serialize_entry(
        "enable_wrapping_detection",
        &gcode.enable_wrapping_detection,
    )?;
    map.serialize_entry("enforce_support_layers", &object.enforce_support_layers)?;
    map.serialize_entry(
        "ensure_vertical_shell_thickness",
        &region.ensure_vertical_shell_thickness,
    )?;
    map.serialize_entry("exclude_object", &print.exclude_object)?;
    map.serialize_entry(
        "extra_perimeters_on_overhangs",
        &region.extra_perimeters_on_overhangs,
    )?;
    map.serialize_entry("extra_solid_infills", &region.extra_solid_infills)?;
    map.serialize_entry(
        "extrusion_rate_smoothing_external_perimeter_only",
        &gcode.extrusion_rate_smoothing_external_perimeter_only,
    )?;
    map.serialize_entry("filename_format", &print.filename_format)?;
    map.serialize_entry("fill_multiline", &region.fill_multiline)?;
    map.serialize_entry("filter_out_gap_fill", &region.filter_out_gap_fill)?;
    map.serialize_entry("first_layer_flow_ratio", &region.first_layer_flow_ratio)?;
    map.serialize_entry("flush_into_infill", &object.flush_into_infill)?;
    map.serialize_entry("flush_into_objects", &object.flush_into_objects)?;
    map.serialize_entry("flush_into_support", &object.flush_into_support)?;
    map.serialize_entry("fuzzy_skin", &region.fuzzy_skin)?;
    map.serialize_entry("fuzzy_skin_first_layer", &region.fuzzy_skin_first_layer)?;
    map.serialize_entry(
        "fuzzy_skin_layers_between_ripple_offset",
        &region.fuzzy_skin_layers_between_ripple_offset,
    )?;
    map.serialize_entry("fuzzy_skin_mode", &region.fuzzy_skin_mode)?;
    map.serialize_entry("fuzzy_skin_noise_type", &region.fuzzy_skin_noise_type)?;
    map.serialize_entry("fuzzy_skin_octaves", &region.fuzzy_skin_octaves)?;
    map.serialize_entry("fuzzy_skin_persistence", &region.fuzzy_skin_persistence)?;
    map.serialize_entry(
        "fuzzy_skin_point_distance",
        &region.fuzzy_skin_point_distance,
    )?;
    map.serialize_entry("fuzzy_skin_ripple_offset", &region.fuzzy_skin_ripple_offset)?;
    map.serialize_entry(
        "fuzzy_skin_ripples_per_layer",
        &region.fuzzy_skin_ripples_per_layer,
    )?;
    map.serialize_entry("fuzzy_skin_scale", &region.fuzzy_skin_scale)?;
    map.serialize_entry("fuzzy_skin_thickness", &region.fuzzy_skin_thickness)?;
    map.serialize_entry("gap_fill_flow_ratio", &region.gap_fill_flow_ratio)?;
    map.serialize_entry("gap_fill_target", &object.gap_fill_target)?;
    map.serialize_entry("gap_infill_speed", &region.gap_infill_speed)?;
    map.serialize_entry("gcode_add_line_number", &gcode.gcode_add_line_number)?;
    map.serialize_entry("gcode_comments", &print.gcode_comments)?;
    map.serialize_entry("gcode_label_objects", &print.gcode_label_objects)?;
    map.serialize_entry("gyroid_optimized", &region.gyroid_optimized)?;
    map.serialize_entry("hole_to_polyhole", &region.hole_to_polyhole)?;
    map.serialize_entry(
        "hole_to_polyhole_threshold",
        &region.hole_to_polyhole_threshold,
    )?;
    map.serialize_entry("hole_to_polyhole_twisted", &region.hole_to_polyhole_twisted)?;
    map.serialize_entry(
        "independent_support_layer_height",
        &print.independent_support_layer_height,
    )?;
    map.serialize_entry("infill_anchor", &region.infill_anchor)?;
    map.serialize_entry("infill_anchor_max", &region.infill_anchor_max)?;
    map.serialize_entry("infill_combination", &region.infill_combination)?;
    map.serialize_entry(
        "infill_combination_max_layer_height",
        &region.infill_combination_max_layer_height,
    )?;
    map.serialize_entry("infill_direction", &region.infill_direction)?;
    map.serialize_entry("infill_jerk", &object.infill_jerk)?;
    map.serialize_entry("infill_lock_depth", &region.infill_lock_depth)?;
    map.serialize_entry("infill_overhang_angle", &region.infill_overhang_angle)?;
    map.serialize_entry("infill_shift_step", &region.infill_shift_step)?;
    map.serialize_entry("infill_wall_overlap", &region.infill_wall_overlap)?;
    map.serialize_entry(
        "initial_layer_acceleration",
        &object.initial_layer_acceleration,
    )?;
    map.serialize_entry(
        "initial_layer_infill_speed",
        &print.initial_layer_infill_speed,
    )?;
    map.serialize_entry("initial_layer_jerk", &object.initial_layer_jerk)?;
    map.serialize_entry("initial_layer_line_width", &print.initial_layer_line_width)?;
    map.serialize_entry(
        "initial_layer_min_bead_width",
        &object.initial_layer_min_bead_width,
    )?;
    map.serialize_entry(
        "initial_layer_print_height",
        &print.initial_layer_print_height,
    )?;
    map.serialize_entry("initial_layer_speed", &print.initial_layer_speed)?;
    map.serialize_entry(
        "initial_layer_travel_acceleration",
        &gcode.initial_layer_travel_acceleration,
    )?;
    map.serialize_entry(
        "initial_layer_travel_jerk",
        &gcode.initial_layer_travel_jerk,
    )?;
    map.serialize_entry(
        "initial_layer_travel_speed",
        &gcode.initial_layer_travel_speed,
    )?;
    map.serialize_entry("inner_wall_acceleration", &object.inner_wall_acceleration)?;
    map.serialize_entry("inner_wall_filament_id", &region.inner_wall_filament_id)?;
    map.serialize_entry("inner_wall_flow_ratio", &region.inner_wall_flow_ratio)?;
    map.serialize_entry("inner_wall_jerk", &object.inner_wall_jerk)?;
    map.serialize_entry("inner_wall_line_width", &region.inner_wall_line_width)?;
    map.serialize_entry("inner_wall_speed", &region.inner_wall_speed)?;
    map.serialize_entry("interface_shells", &object.interface_shells)?;
    map.serialize_entry("interlocking_beam", &object.interlocking_beam)?;
    map.serialize_entry(
        "interlocking_beam_layer_count",
        &object.interlocking_beam_layer_count,
    )?;
    map.serialize_entry("interlocking_beam_width", &object.interlocking_beam_width)?;
    map.serialize_entry(
        "interlocking_boundary_avoidance",
        &object.interlocking_boundary_avoidance,
    )?;
    map.serialize_entry("interlocking_depth", &object.interlocking_depth)?;
    map.serialize_entry("interlocking_orientation", &object.interlocking_orientation)?;
    Ok(())
}
