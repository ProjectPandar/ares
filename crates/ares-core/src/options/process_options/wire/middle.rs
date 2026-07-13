use serde::ser::SerializeMap;

use super::super::ProcessOptions;

pub(super) fn serialize_entries<M>(map: &mut M, process: &ProcessOptions) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    let ProcessOptions {
        gcode,
        ironing_expansion,
        object,
        print,
        region,
    } = process;
    map.serialize_entry("internal_bridge_angle", &region.internal_bridge_angle)?;
    map.serialize_entry("internal_bridge_density", &object.internal_bridge_density)?;
    map.serialize_entry("internal_bridge_flow", &region.internal_bridge_flow)?;
    map.serialize_entry("internal_bridge_speed", &region.internal_bridge_speed)?;
    map.serialize_entry(
        "internal_solid_filament_id",
        &region.internal_solid_filament_id,
    )?;
    map.serialize_entry(
        "internal_solid_infill_acceleration",
        &object.internal_solid_infill_acceleration,
    )?;
    map.serialize_entry(
        "internal_solid_infill_flow_ratio",
        &region.internal_solid_infill_flow_ratio,
    )?;
    map.serialize_entry(
        "internal_solid_infill_line_width",
        &region.internal_solid_infill_line_width,
    )?;
    map.serialize_entry(
        "internal_solid_infill_pattern",
        &region.internal_solid_infill_pattern,
    )?;
    map.serialize_entry(
        "internal_solid_infill_speed",
        &region.internal_solid_infill_speed,
    )?;
    map.serialize_entry("ironing_angle", &region.ironing_angle)?;
    map.serialize_entry("ironing_angle_fixed", &region.ironing_angle_fixed)?;
    map.serialize_entry("ironing_expansion", ironing_expansion)?;
    map.serialize_entry("ironing_flow", &region.ironing_flow)?;
    map.serialize_entry("ironing_inset", &region.ironing_inset)?;
    map.serialize_entry("ironing_pattern", &region.ironing_pattern)?;
    map.serialize_entry("ironing_spacing", &region.ironing_spacing)?;
    map.serialize_entry("ironing_speed", &region.ironing_speed)?;
    map.serialize_entry("ironing_type", &region.ironing_type)?;
    map.serialize_entry("is_infill_first", &region.is_infill_first)?;
    map.serialize_entry("lateral_lattice_angle_1", &region.lateral_lattice_angle_1)?;
    map.serialize_entry("lateral_lattice_angle_2", &region.lateral_lattice_angle_2)?;
    map.serialize_entry("layer_height", &object.layer_height)?;
    map.serialize_entry("lightning_overhang_angle", &region.lightning_overhang_angle)?;
    map.serialize_entry("lightning_prune_angle", &region.lightning_prune_angle)?;
    map.serialize_entry(
        "lightning_straightening_angle",
        &region.lightning_straightening_angle,
    )?;
    map.serialize_entry("line_width", &object.line_width)?;
    map.serialize_entry("make_overhang_printable", &region.make_overhang_printable)?;
    map.serialize_entry(
        "make_overhang_printable_angle",
        &object.make_overhang_printable_angle,
    )?;
    map.serialize_entry(
        "make_overhang_printable_hole_size",
        &object.make_overhang_printable_hole_size,
    )?;
    map.serialize_entry("max_bridge_length", &object.max_bridge_length)?;
    map.serialize_entry(
        "max_travel_detour_distance",
        &print.max_travel_detour_distance,
    )?;
    map.serialize_entry(
        "max_volumetric_extrusion_rate_slope",
        &gcode.max_volumetric_extrusion_rate_slope,
    )?;
    map.serialize_entry(
        "max_volumetric_extrusion_rate_slope_segment_length",
        &gcode.max_volumetric_extrusion_rate_slope_segment_length,
    )?;
    map.serialize_entry("min_bead_width", &object.min_bead_width)?;
    map.serialize_entry("min_feature_size", &object.min_feature_size)?;
    map.serialize_entry("min_length_factor", &object.min_length_factor)?;
    map.serialize_entry("min_skirt_length", &print.min_skirt_length)?;
    map.serialize_entry("min_width_top_surface", &region.min_width_top_surface)?;
    map.serialize_entry(
        "minimum_sparse_infill_area",
        &region.minimum_sparse_infill_area,
    )?;
    map.serialize_entry(
        "mmu_segmented_region_interlocking_depth",
        &object.mmu_segmented_region_interlocking_depth,
    )?;
    map.serialize_entry(
        "mmu_segmented_region_max_width",
        &object.mmu_segmented_region_max_width,
    )?;
    map.serialize_entry("notes", &print.notes)?;
    map.serialize_entry(
        "only_one_wall_first_layer",
        &region.only_one_wall_first_layer,
    )?;
    map.serialize_entry("only_one_wall_top", &region.only_one_wall_top)?;
    map.serialize_entry("ooze_prevention", &print.ooze_prevention)?;
    map.serialize_entry("outer_wall_acceleration", &object.outer_wall_acceleration)?;
    map.serialize_entry("outer_wall_filament_id", &region.outer_wall_filament_id)?;
    map.serialize_entry("outer_wall_flow_ratio", &region.outer_wall_flow_ratio)?;
    map.serialize_entry("outer_wall_jerk", &object.outer_wall_jerk)?;
    map.serialize_entry("outer_wall_line_width", &region.outer_wall_line_width)?;
    map.serialize_entry("outer_wall_speed", &region.outer_wall_speed)?;
    map.serialize_entry("overhang_1_4_speed", &region.overhang_1_4_speed)?;
    map.serialize_entry("overhang_2_4_speed", &region.overhang_2_4_speed)?;
    map.serialize_entry("overhang_3_4_speed", &region.overhang_3_4_speed)?;
    map.serialize_entry("overhang_4_4_speed", &region.overhang_4_4_speed)?;
    map.serialize_entry("overhang_flow_ratio", &region.overhang_flow_ratio)?;
    map.serialize_entry("overhang_reverse", &region.overhang_reverse)?;
    map.serialize_entry(
        "overhang_reverse_internal_only",
        &region.overhang_reverse_internal_only,
    )?;
    map.serialize_entry(
        "overhang_reverse_threshold",
        &region.overhang_reverse_threshold,
    )?;
    map.serialize_entry("post_process", &print.post_process)?;
    map.serialize_entry("precise_outer_wall", &region.precise_outer_wall)?;
    map.serialize_entry("precise_z_height", &object.precise_z_height)?;
    map.serialize_entry("preheat_steps", &print.preheat_steps)?;
    map.serialize_entry("preheat_time", &print.preheat_time)?;
    map.serialize_entry("prime_tower_brim_width", &print.prime_tower_brim_width)?;
    map.serialize_entry(
        "prime_tower_enable_framework",
        &print.prime_tower_enable_framework,
    )?;
    map.serialize_entry("prime_tower_flat_ironing", &print.prime_tower_flat_ironing)?;
    map.serialize_entry("prime_tower_infill_gap", &print.prime_tower_infill_gap)?;
    map.serialize_entry("prime_tower_skip_points", &print.prime_tower_skip_points)?;
    map.serialize_entry("prime_tower_width", &print.prime_tower_width)?;
    map.serialize_entry("prime_volume", &print.prime_volume)?;
    map.serialize_entry("print_extruder_id", &region.print_extruder_id)?;
    map.serialize_entry("print_extruder_variant", &region.print_extruder_variant)?;
    map.serialize_entry("print_flow_ratio", &region.print_flow_ratio)?;
    map.serialize_entry("print_order", &print.print_order)?;
    map.serialize_entry("print_sequence", &print.print_sequence)?;
    map.serialize_entry(
        "process_change_extrusion_role_gcode",
        &gcode.process_change_extrusion_role_gcode,
    )?;
    map.serialize_entry("raft_contact_distance", &object.raft_contact_distance)?;
    map.serialize_entry("raft_expansion", &object.raft_expansion)?;
    map.serialize_entry("raft_first_layer_density", &object.raft_first_layer_density)?;
    map.serialize_entry(
        "raft_first_layer_expansion",
        &object.raft_first_layer_expansion,
    )?;
    map.serialize_entry("raft_layers", &object.raft_layers)?;
    map.serialize_entry("reduce_crossing_wall", &print.reduce_crossing_wall)?;
    map.serialize_entry("reduce_infill_retraction", &print.reduce_infill_retraction)?;
    map.serialize_entry("relative_bridge_angle", &region.relative_bridge_angle)?;
    map.serialize_entry("resolution", &print.resolution)?;
    map.serialize_entry("role_based_wipe_speed", &region.role_based_wipe_speed)?;
    map.serialize_entry("scarf_angle_threshold", &region.scarf_angle_threshold)?;
    map.serialize_entry("scarf_joint_flow_ratio", &region.scarf_joint_flow_ratio)?;
    map.serialize_entry("scarf_joint_speed", &region.scarf_joint_speed)?;
    map.serialize_entry("scarf_overhang_threshold", &region.scarf_overhang_threshold)?;
    map.serialize_entry("seam_gap", &region.seam_gap)?;
    map.serialize_entry("seam_position", &object.seam_position)?;
    map.serialize_entry("seam_slope_conditional", &region.seam_slope_conditional)?;
    map.serialize_entry("seam_slope_entire_loop", &region.seam_slope_entire_loop)?;
    map.serialize_entry("seam_slope_inner_walls", &region.seam_slope_inner_walls)?;
    map.serialize_entry("seam_slope_min_length", &region.seam_slope_min_length)?;
    map.serialize_entry("seam_slope_start_height", &region.seam_slope_start_height)?;
    map.serialize_entry("seam_slope_steps", &region.seam_slope_steps)?;
    map.serialize_entry("seam_slope_type", &region.seam_slope_type)?;
    map.serialize_entry("set_other_flow_ratios", &object.set_other_flow_ratios)?;
    map.serialize_entry(
        "single_extruder_multi_material_priming",
        &gcode.single_extruder_multi_material_priming,
    )?;
    map.serialize_entry("single_loop_draft_shield", &print.single_loop_draft_shield)?;
    map.serialize_entry("skeleton_infill_density", &region.skeleton_infill_density)?;
    map.serialize_entry(
        "skeleton_infill_line_width",
        &region.skeleton_infill_line_width,
    )?;
    map.serialize_entry("skin_infill_density", &region.skin_infill_density)?;
    map.serialize_entry("skin_infill_depth", &region.skin_infill_depth)?;
    map.serialize_entry("skin_infill_line_width", &region.skin_infill_line_width)?;
    map.serialize_entry("skirt_distance", &print.skirt_distance)?;
    map.serialize_entry("skirt_height", &print.skirt_height)?;
    map.serialize_entry("skirt_loops", &print.skirt_loops)?;
    map.serialize_entry("skirt_speed", &print.skirt_speed)?;
    map.serialize_entry("skirt_start_angle", &object.skirt_start_angle)?;
    map.serialize_entry("skirt_type", &print.skirt_type)?;
    map.serialize_entry("slice_closing_radius", &object.slice_closing_radius)?;
    map.serialize_entry("slicing_mode", &object.slicing_mode)?;
    map.serialize_entry("slow_down_layers", &print.slow_down_layers)?;
    map.serialize_entry(
        "slowdown_for_curled_perimeters",
        &region.slowdown_for_curled_perimeters,
    )?;
    map.serialize_entry(
        "small_area_infill_flow_compensation",
        &region.small_area_infill_flow_compensation,
    )?;
    map.serialize_entry(
        "small_area_infill_flow_compensation_model",
        &gcode.small_area_infill_flow_compensation_model,
    )?;
    map.serialize_entry("small_perimeter_speed", &region.small_perimeter_speed)?;
    map.serialize_entry(
        "small_perimeter_threshold",
        &region.small_perimeter_threshold,
    )?;
    map.serialize_entry("solid_infill_direction", &region.solid_infill_direction)?;
    Ok(())
}
