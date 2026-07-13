use serde::ser::SerializeMap;

use super::super::{ProcessObjectSourceOptions, ProcessRegionSourceOptions};

pub(super) fn serialize_entries<M>(
    map: &mut M,
    object: &ProcessObjectSourceOptions,
    region: &ProcessRegionSourceOptions,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "solid_infill_rotate_template",
        &region.solid_infill_rotate_template,
    )?;
    map.serialize_entry(
        "sparse_infill_acceleration",
        &object.sparse_infill_acceleration,
    )?;
    map.serialize_entry("sparse_infill_density", &region.sparse_infill_density)?;
    map.serialize_entry(
        "sparse_infill_filament_id",
        &region.sparse_infill_filament_id,
    )?;
    map.serialize_entry("sparse_infill_flow_ratio", &region.sparse_infill_flow_ratio)?;
    map.serialize_entry("sparse_infill_line_width", &region.sparse_infill_line_width)?;
    map.serialize_entry("sparse_infill_pattern", &region.sparse_infill_pattern)?;
    map.serialize_entry(
        "sparse_infill_rotate_template",
        &region.sparse_infill_rotate_template,
    )?;
    map.serialize_entry("sparse_infill_speed", &region.sparse_infill_speed)?;
    map.serialize_entry("staggered_inner_seams", &object.staggered_inner_seams)?;
    map.serialize_entry("support_angle", &object.support_angle)?;
    map.serialize_entry("support_base_pattern", &object.support_base_pattern)?;
    map.serialize_entry(
        "support_base_pattern_spacing",
        &object.support_base_pattern_spacing,
    )?;
    map.serialize_entry(
        "support_bottom_interface_spacing",
        &object.support_bottom_interface_spacing,
    )?;
    map.serialize_entry(
        "support_bottom_z_distance",
        &object.support_bottom_z_distance,
    )?;
    map.serialize_entry(
        "support_critical_regions_only",
        &object.support_critical_regions_only,
    )?;
    map.serialize_entry("support_expansion", &object.support_expansion)?;
    map.serialize_entry("support_filament", &object.support_filament)?;
    map.serialize_entry("support_flow_ratio", &object.support_flow_ratio)?;
    map.serialize_entry(
        "support_interface_bottom_layers",
        &object.support_interface_bottom_layers,
    )?;
    map.serialize_entry(
        "support_interface_filament",
        &object.support_interface_filament,
    )?;
    map.serialize_entry(
        "support_interface_flow_ratio",
        &object.support_interface_flow_ratio,
    )?;
    map.serialize_entry(
        "support_interface_loop_pattern",
        &object.support_interface_loop_pattern,
    )?;
    map.serialize_entry(
        "support_interface_not_for_body",
        &object.support_interface_not_for_body,
    )?;
    map.serialize_entry(
        "support_interface_pattern",
        &object.support_interface_pattern,
    )?;
    map.serialize_entry(
        "support_interface_spacing",
        &object.support_interface_spacing,
    )?;
    map.serialize_entry("support_interface_speed", &object.support_interface_speed)?;
    map.serialize_entry(
        "support_interface_top_layers",
        &object.support_interface_top_layers,
    )?;
    map.serialize_entry("support_ironing", &object.support_ironing)?;
    map.serialize_entry("support_ironing_flow", &object.support_ironing_flow)?;
    map.serialize_entry("support_ironing_pattern", &object.support_ironing_pattern)?;
    map.serialize_entry("support_ironing_spacing", &object.support_ironing_spacing)?;
    map.serialize_entry("support_line_width", &object.support_line_width)?;
    map.serialize_entry(
        "support_object_first_layer_gap",
        &object.support_object_first_layer_gap,
    )?;
    map.serialize_entry(
        "support_object_xy_distance",
        &object.support_object_xy_distance,
    )?;
    map.serialize_entry(
        "support_on_build_plate_only",
        &object.support_on_build_plate_only,
    )?;
    map.serialize_entry(
        "support_remove_small_overhang",
        &object.support_remove_small_overhang,
    )?;
    map.serialize_entry("support_speed", &object.support_speed)?;
    map.serialize_entry("support_style", &object.support_style)?;
    map.serialize_entry("support_threshold_angle", &object.support_threshold_angle)?;
    map.serialize_entry(
        "support_threshold_overlap",
        &object.support_threshold_overlap,
    )?;
    map.serialize_entry("support_top_z_distance", &object.support_top_z_distance)?;
    map.serialize_entry("support_type", &object.support_type)?;
    map.serialize_entry("symmetric_infill_y_axis", &region.symmetric_infill_y_axis)?;
    map.serialize_entry("thick_bridges", &object.thick_bridges)?;
    map.serialize_entry("thick_internal_bridges", &object.thick_internal_bridges)?;
    map.serialize_entry(
        "top_bottom_infill_wall_overlap",
        &region.top_bottom_infill_wall_overlap,
    )?;
    map.serialize_entry("top_shell_layers", &region.top_shell_layers)?;
    map.serialize_entry("top_shell_thickness", &region.top_shell_thickness)?;
    map.serialize_entry(
        "top_solid_infill_flow_ratio",
        &region.top_solid_infill_flow_ratio,
    )?;
    map.serialize_entry("top_surface_acceleration", &object.top_surface_acceleration)?;
    map.serialize_entry("top_surface_density", &region.top_surface_density)?;
    map.serialize_entry("top_surface_filament_id", &region.top_surface_filament_id)?;
    map.serialize_entry("top_surface_jerk", &object.top_surface_jerk)?;
    map.serialize_entry("top_surface_line_width", &region.top_surface_line_width)?;
    map.serialize_entry("top_surface_pattern", &region.top_surface_pattern)?;
    map.serialize_entry("top_surface_speed", &region.top_surface_speed)?;
    map.serialize_entry("travel_acceleration", &object.travel_acceleration)?;
    map.serialize_entry("travel_jerk", &object.travel_jerk)?;
    map.serialize_entry("tree_support_angle_slow", &object.tree_support_angle_slow)?;
    map.serialize_entry("tree_support_auto_brim", &object.tree_support_auto_brim)?;
    map.serialize_entry(
        "tree_support_branch_angle",
        &object.tree_support_branch_angle,
    )?;
    map.serialize_entry(
        "tree_support_branch_angle_organic",
        &object.tree_support_branch_angle_organic,
    )?;
    map.serialize_entry(
        "tree_support_branch_diameter",
        &object.tree_support_branch_diameter,
    )?;
    map.serialize_entry(
        "tree_support_branch_diameter_angle",
        &object.tree_support_branch_diameter_angle,
    )?;
    map.serialize_entry(
        "tree_support_branch_diameter_organic",
        &object.tree_support_branch_diameter_organic,
    )?;
    map.serialize_entry(
        "tree_support_branch_distance",
        &object.tree_support_branch_distance,
    )?;
    map.serialize_entry(
        "tree_support_branch_distance_organic",
        &object.tree_support_branch_distance_organic,
    )?;
    map.serialize_entry("tree_support_brim_width", &object.tree_support_brim_width)?;
    map.serialize_entry(
        "tree_support_tip_diameter",
        &object.tree_support_tip_diameter,
    )?;
    map.serialize_entry("tree_support_top_rate", &object.tree_support_top_rate)?;
    map.serialize_entry("tree_support_wall_count", &object.tree_support_wall_count)?;
    map.serialize_entry("wall_direction", &region.wall_direction)?;
    map.serialize_entry("wall_distribution_count", &object.wall_distribution_count)?;
    map.serialize_entry("wall_generator", &object.wall_generator)?;
    map.serialize_entry("wall_loops", &region.wall_loops)?;
    map.serialize_entry("wall_maximum_deviation", &object.wall_maximum_deviation)?;
    map.serialize_entry("wall_maximum_resolution", &object.wall_maximum_resolution)?;
    map.serialize_entry("wall_sequence", &region.wall_sequence)?;
    map.serialize_entry("wall_transition_angle", &object.wall_transition_angle)?;
    map.serialize_entry(
        "wall_transition_filter_deviation",
        &object.wall_transition_filter_deviation,
    )?;
    map.serialize_entry("wall_transition_length", &object.wall_transition_length)?;
    map.serialize_entry(
        "wipe_before_external_loop",
        &region.wipe_before_external_loop,
    )?;
    map.serialize_entry("wipe_on_loops", &region.wipe_on_loops)?;
    map.serialize_entry("wipe_speed", &region.wipe_speed)?;
    map.serialize_entry("xy_contour_compensation", &object.xy_contour_compensation)?;
    map.serialize_entry("xy_hole_compensation", &object.xy_hole_compensation)?;
    map.serialize_entry(
        "zaa_dont_alternate_fill_direction",
        &region.zaa_dont_alternate_fill_direction,
    )?;
    map.serialize_entry("zaa_enabled", &region.zaa_enabled)?;
    map.serialize_entry("zaa_min_z", &region.zaa_min_z)?;
    map.serialize_entry(
        "zaa_minimize_perimeter_height",
        &region.zaa_minimize_perimeter_height,
    )?;
    Ok(())
}
