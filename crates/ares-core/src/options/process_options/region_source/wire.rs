use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProcessRegionSourceOptions;

impl Serialize for ProcessRegionSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(149))?;
        map.serialize_entry(
            "align_infill_direction_to_model",
            &self.align_infill_direction_to_model,
        )?;
        map.serialize_entry("alternate_extra_wall", &self.alternate_extra_wall)?;
        map.serialize_entry("bottom_shell_layers", &self.bottom_shell_layers)?;
        map.serialize_entry("bottom_shell_thickness", &self.bottom_shell_thickness)?;
        map.serialize_entry(
            "bottom_solid_infill_flow_ratio",
            &self.bottom_solid_infill_flow_ratio,
        )?;
        map.serialize_entry("bottom_surface_density", &self.bottom_surface_density)?;
        map.serialize_entry(
            "bottom_surface_filament_id",
            &self.bottom_surface_filament_id,
        )?;
        map.serialize_entry("bottom_surface_pattern", &self.bottom_surface_pattern)?;
        map.serialize_entry("bridge_angle", &self.bridge_angle)?;
        map.serialize_entry("bridge_density", &self.bridge_density)?;
        map.serialize_entry("bridge_flow", &self.bridge_flow)?;
        map.serialize_entry("bridge_line_width", &self.bridge_line_width)?;
        map.serialize_entry("bridge_speed", &self.bridge_speed)?;
        map.serialize_entry("counterbore_hole_bridging", &self.counterbore_hole_bridging)?;
        map.serialize_entry("detect_overhang_wall", &self.detect_overhang_wall)?;
        map.serialize_entry("detect_thin_wall", &self.detect_thin_wall)?;
        map.serialize_entry("enable_overhang_speed", &self.enable_overhang_speed)?;
        map.serialize_entry(
            "ensure_vertical_shell_thickness",
            &self.ensure_vertical_shell_thickness,
        )?;
        map.serialize_entry(
            "extra_perimeters_on_overhangs",
            &self.extra_perimeters_on_overhangs,
        )?;
        map.serialize_entry("extra_solid_infills", &self.extra_solid_infills)?;
        map.serialize_entry("fill_multiline", &self.fill_multiline)?;
        map.serialize_entry("filter_out_gap_fill", &self.filter_out_gap_fill)?;
        map.serialize_entry("first_layer_flow_ratio", &self.first_layer_flow_ratio)?;
        map.serialize_entry("fuzzy_skin", &self.fuzzy_skin)?;
        map.serialize_entry("fuzzy_skin_first_layer", &self.fuzzy_skin_first_layer)?;
        map.serialize_entry(
            "fuzzy_skin_layers_between_ripple_offset",
            &self.fuzzy_skin_layers_between_ripple_offset,
        )?;
        map.serialize_entry("fuzzy_skin_mode", &self.fuzzy_skin_mode)?;
        map.serialize_entry("fuzzy_skin_noise_type", &self.fuzzy_skin_noise_type)?;
        map.serialize_entry("fuzzy_skin_octaves", &self.fuzzy_skin_octaves)?;
        map.serialize_entry("fuzzy_skin_persistence", &self.fuzzy_skin_persistence)?;
        map.serialize_entry("fuzzy_skin_point_distance", &self.fuzzy_skin_point_distance)?;
        map.serialize_entry("fuzzy_skin_ripple_offset", &self.fuzzy_skin_ripple_offset)?;
        map.serialize_entry(
            "fuzzy_skin_ripples_per_layer",
            &self.fuzzy_skin_ripples_per_layer,
        )?;
        map.serialize_entry("fuzzy_skin_scale", &self.fuzzy_skin_scale)?;
        map.serialize_entry("fuzzy_skin_thickness", &self.fuzzy_skin_thickness)?;
        map.serialize_entry("gap_fill_flow_ratio", &self.gap_fill_flow_ratio)?;
        map.serialize_entry("gap_infill_speed", &self.gap_infill_speed)?;
        map.serialize_entry("gyroid_optimized", &self.gyroid_optimized)?;
        map.serialize_entry("hole_to_polyhole", &self.hole_to_polyhole)?;
        map.serialize_entry(
            "hole_to_polyhole_threshold",
            &self.hole_to_polyhole_threshold,
        )?;
        map.serialize_entry("hole_to_polyhole_twisted", &self.hole_to_polyhole_twisted)?;
        map.serialize_entry("infill_anchor", &self.infill_anchor)?;
        map.serialize_entry("infill_anchor_max", &self.infill_anchor_max)?;
        map.serialize_entry("infill_combination", &self.infill_combination)?;
        map.serialize_entry(
            "infill_combination_max_layer_height",
            &self.infill_combination_max_layer_height,
        )?;
        map.serialize_entry("infill_direction", &self.infill_direction)?;
        map.serialize_entry("infill_lock_depth", &self.infill_lock_depth)?;
        map.serialize_entry("infill_overhang_angle", &self.infill_overhang_angle)?;
        map.serialize_entry("infill_shift_step", &self.infill_shift_step)?;
        map.serialize_entry("infill_wall_overlap", &self.infill_wall_overlap)?;
        map.serialize_entry("inner_wall_filament_id", &self.inner_wall_filament_id)?;
        map.serialize_entry("inner_wall_flow_ratio", &self.inner_wall_flow_ratio)?;
        map.serialize_entry("inner_wall_line_width", &self.inner_wall_line_width)?;
        map.serialize_entry("inner_wall_speed", &self.inner_wall_speed)?;
        map.serialize_entry("internal_bridge_angle", &self.internal_bridge_angle)?;
        map.serialize_entry("internal_bridge_flow", &self.internal_bridge_flow)?;
        map.serialize_entry("internal_bridge_speed", &self.internal_bridge_speed)?;
        map.serialize_entry(
            "internal_solid_filament_id",
            &self.internal_solid_filament_id,
        )?;
        map.serialize_entry(
            "internal_solid_infill_flow_ratio",
            &self.internal_solid_infill_flow_ratio,
        )?;
        map.serialize_entry(
            "internal_solid_infill_line_width",
            &self.internal_solid_infill_line_width,
        )?;
        map.serialize_entry(
            "internal_solid_infill_pattern",
            &self.internal_solid_infill_pattern,
        )?;
        map.serialize_entry(
            "internal_solid_infill_speed",
            &self.internal_solid_infill_speed,
        )?;
        map.serialize_entry("ironing_angle", &self.ironing_angle)?;
        map.serialize_entry("ironing_angle_fixed", &self.ironing_angle_fixed)?;
        map.serialize_entry("ironing_flow", &self.ironing_flow)?;
        map.serialize_entry("ironing_inset", &self.ironing_inset)?;
        map.serialize_entry("ironing_pattern", &self.ironing_pattern)?;
        map.serialize_entry("ironing_spacing", &self.ironing_spacing)?;
        map.serialize_entry("ironing_speed", &self.ironing_speed)?;
        map.serialize_entry("ironing_type", &self.ironing_type)?;
        map.serialize_entry("is_infill_first", &self.is_infill_first)?;
        map.serialize_entry("lateral_lattice_angle_1", &self.lateral_lattice_angle_1)?;
        map.serialize_entry("lateral_lattice_angle_2", &self.lateral_lattice_angle_2)?;
        map.serialize_entry("lightning_overhang_angle", &self.lightning_overhang_angle)?;
        map.serialize_entry("lightning_prune_angle", &self.lightning_prune_angle)?;
        map.serialize_entry(
            "lightning_straightening_angle",
            &self.lightning_straightening_angle,
        )?;
        map.serialize_entry("make_overhang_printable", &self.make_overhang_printable)?;
        map.serialize_entry("min_width_top_surface", &self.min_width_top_surface)?;
        map.serialize_entry(
            "minimum_sparse_infill_area",
            &self.minimum_sparse_infill_area,
        )?;
        map.serialize_entry("only_one_wall_first_layer", &self.only_one_wall_first_layer)?;
        map.serialize_entry("only_one_wall_top", &self.only_one_wall_top)?;
        map.serialize_entry("outer_wall_filament_id", &self.outer_wall_filament_id)?;
        map.serialize_entry("outer_wall_flow_ratio", &self.outer_wall_flow_ratio)?;
        map.serialize_entry("outer_wall_line_width", &self.outer_wall_line_width)?;
        map.serialize_entry("outer_wall_speed", &self.outer_wall_speed)?;
        map.serialize_entry("overhang_1_4_speed", &self.overhang_1_4_speed)?;
        map.serialize_entry("overhang_2_4_speed", &self.overhang_2_4_speed)?;
        map.serialize_entry("overhang_3_4_speed", &self.overhang_3_4_speed)?;
        map.serialize_entry("overhang_4_4_speed", &self.overhang_4_4_speed)?;
        map.serialize_entry("overhang_flow_ratio", &self.overhang_flow_ratio)?;
        map.serialize_entry("overhang_reverse", &self.overhang_reverse)?;
        map.serialize_entry(
            "overhang_reverse_internal_only",
            &self.overhang_reverse_internal_only,
        )?;
        map.serialize_entry(
            "overhang_reverse_threshold",
            &self.overhang_reverse_threshold,
        )?;
        map.serialize_entry("precise_outer_wall", &self.precise_outer_wall)?;
        map.serialize_entry("print_extruder_id", &self.print_extruder_id)?;
        map.serialize_entry("print_extruder_variant", &self.print_extruder_variant)?;
        map.serialize_entry("print_flow_ratio", &self.print_flow_ratio)?;
        map.serialize_entry("relative_bridge_angle", &self.relative_bridge_angle)?;
        map.serialize_entry("role_based_wipe_speed", &self.role_based_wipe_speed)?;
        map.serialize_entry("scarf_angle_threshold", &self.scarf_angle_threshold)?;
        map.serialize_entry("scarf_joint_flow_ratio", &self.scarf_joint_flow_ratio)?;
        map.serialize_entry("scarf_joint_speed", &self.scarf_joint_speed)?;
        map.serialize_entry("scarf_overhang_threshold", &self.scarf_overhang_threshold)?;
        map.serialize_entry("seam_gap", &self.seam_gap)?;
        map.serialize_entry("seam_slope_conditional", &self.seam_slope_conditional)?;
        map.serialize_entry("seam_slope_entire_loop", &self.seam_slope_entire_loop)?;
        map.serialize_entry("seam_slope_inner_walls", &self.seam_slope_inner_walls)?;
        map.serialize_entry("seam_slope_min_length", &self.seam_slope_min_length)?;
        map.serialize_entry("seam_slope_start_height", &self.seam_slope_start_height)?;
        map.serialize_entry("seam_slope_steps", &self.seam_slope_steps)?;
        map.serialize_entry("seam_slope_type", &self.seam_slope_type)?;
        map.serialize_entry("skeleton_infill_density", &self.skeleton_infill_density)?;
        map.serialize_entry(
            "skeleton_infill_line_width",
            &self.skeleton_infill_line_width,
        )?;
        map.serialize_entry("skin_infill_density", &self.skin_infill_density)?;
        map.serialize_entry("skin_infill_depth", &self.skin_infill_depth)?;
        map.serialize_entry("skin_infill_line_width", &self.skin_infill_line_width)?;
        map.serialize_entry(
            "slowdown_for_curled_perimeters",
            &self.slowdown_for_curled_perimeters,
        )?;
        map.serialize_entry(
            "small_area_infill_flow_compensation",
            &self.small_area_infill_flow_compensation,
        )?;
        map.serialize_entry("small_perimeter_speed", &self.small_perimeter_speed)?;
        map.serialize_entry("small_perimeter_threshold", &self.small_perimeter_threshold)?;
        map.serialize_entry("solid_infill_direction", &self.solid_infill_direction)?;
        map.serialize_entry(
            "solid_infill_rotate_template",
            &self.solid_infill_rotate_template,
        )?;
        map.serialize_entry("sparse_infill_density", &self.sparse_infill_density)?;
        map.serialize_entry("sparse_infill_filament_id", &self.sparse_infill_filament_id)?;
        map.serialize_entry("sparse_infill_flow_ratio", &self.sparse_infill_flow_ratio)?;
        map.serialize_entry("sparse_infill_line_width", &self.sparse_infill_line_width)?;
        map.serialize_entry("sparse_infill_pattern", &self.sparse_infill_pattern)?;
        map.serialize_entry(
            "sparse_infill_rotate_template",
            &self.sparse_infill_rotate_template,
        )?;
        map.serialize_entry("sparse_infill_speed", &self.sparse_infill_speed)?;
        map.serialize_entry("symmetric_infill_y_axis", &self.symmetric_infill_y_axis)?;
        map.serialize_entry(
            "top_bottom_infill_wall_overlap",
            &self.top_bottom_infill_wall_overlap,
        )?;
        map.serialize_entry("top_shell_layers", &self.top_shell_layers)?;
        map.serialize_entry("top_shell_thickness", &self.top_shell_thickness)?;
        map.serialize_entry(
            "top_solid_infill_flow_ratio",
            &self.top_solid_infill_flow_ratio,
        )?;
        map.serialize_entry("top_surface_density", &self.top_surface_density)?;
        map.serialize_entry("top_surface_filament_id", &self.top_surface_filament_id)?;
        map.serialize_entry("top_surface_line_width", &self.top_surface_line_width)?;
        map.serialize_entry("top_surface_pattern", &self.top_surface_pattern)?;
        map.serialize_entry("top_surface_speed", &self.top_surface_speed)?;
        map.serialize_entry("wall_direction", &self.wall_direction)?;
        map.serialize_entry("wall_loops", &self.wall_loops)?;
        map.serialize_entry("wall_sequence", &self.wall_sequence)?;
        map.serialize_entry("wipe_before_external_loop", &self.wipe_before_external_loop)?;
        map.serialize_entry("wipe_on_loops", &self.wipe_on_loops)?;
        map.serialize_entry("wipe_speed", &self.wipe_speed)?;
        map.serialize_entry(
            "zaa_dont_alternate_fill_direction",
            &self.zaa_dont_alternate_fill_direction,
        )?;
        map.serialize_entry("zaa_enabled", &self.zaa_enabled)?;
        map.serialize_entry("zaa_min_z", &self.zaa_min_z)?;
        map.serialize_entry(
            "zaa_minimize_perimeter_height",
            &self.zaa_minimize_perimeter_height,
        )?;
        map.end()
    }
}
