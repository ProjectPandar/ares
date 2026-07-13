use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProcessObjectSourceOptions;

impl Serialize for ProcessObjectSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(126))?;
        map.serialize_entry("bridge_acceleration", &self.bridge_acceleration)?;
        map.serialize_entry("bridge_no_support", &self.bridge_no_support)?;
        map.serialize_entry(
            "brim_ears_detection_length",
            &self.brim_ears_detection_length,
        )?;
        map.serialize_entry("brim_ears_max_angle", &self.brim_ears_max_angle)?;
        map.serialize_entry("brim_flow_ratio", &self.brim_flow_ratio)?;
        map.serialize_entry("brim_object_gap", &self.brim_object_gap)?;
        map.serialize_entry("brim_type", &self.brim_type)?;
        map.serialize_entry("brim_use_efc_outline", &self.brim_use_efc_outline)?;
        map.serialize_entry("brim_width", &self.brim_width)?;
        map.serialize_entry(
            "calib_flowrate_topinfill_special_order",
            &self.calib_flowrate_topinfill_special_order,
        )?;
        map.serialize_entry("default_acceleration", &self.default_acceleration)?;
        map.serialize_entry("default_jerk", &self.default_jerk)?;
        map.serialize_entry(
            "default_junction_deviation",
            &self.default_junction_deviation,
        )?;
        map.serialize_entry(
            "detect_narrow_internal_solid_infill",
            &self.detect_narrow_internal_solid_infill,
        )?;
        map.serialize_entry(
            "dont_filter_internal_bridges",
            &self.dont_filter_internal_bridges,
        )?;
        map.serialize_entry("elefant_foot_compensation", &self.elefant_foot_compensation)?;
        map.serialize_entry(
            "elefant_foot_compensation_layers",
            &self.elefant_foot_compensation_layers,
        )?;
        map.serialize_entry(
            "elefant_foot_layers_density",
            &self.elefant_foot_layers_density,
        )?;
        map.serialize_entry("enable_extra_bridge_layer", &self.enable_extra_bridge_layer)?;
        map.serialize_entry("enable_support", &self.enable_support)?;
        map.serialize_entry("enforce_support_layers", &self.enforce_support_layers)?;
        map.serialize_entry("flush_into_infill", &self.flush_into_infill)?;
        map.serialize_entry("flush_into_objects", &self.flush_into_objects)?;
        map.serialize_entry("flush_into_support", &self.flush_into_support)?;
        map.serialize_entry("gap_fill_target", &self.gap_fill_target)?;
        map.serialize_entry("infill_jerk", &self.infill_jerk)?;
        map.serialize_entry(
            "initial_layer_acceleration",
            &self.initial_layer_acceleration,
        )?;
        map.serialize_entry("initial_layer_jerk", &self.initial_layer_jerk)?;
        map.serialize_entry(
            "initial_layer_min_bead_width",
            &self.initial_layer_min_bead_width,
        )?;
        map.serialize_entry("inner_wall_acceleration", &self.inner_wall_acceleration)?;
        map.serialize_entry("inner_wall_jerk", &self.inner_wall_jerk)?;
        map.serialize_entry("interface_shells", &self.interface_shells)?;
        map.serialize_entry("interlocking_beam", &self.interlocking_beam)?;
        map.serialize_entry(
            "interlocking_beam_layer_count",
            &self.interlocking_beam_layer_count,
        )?;
        map.serialize_entry("interlocking_beam_width", &self.interlocking_beam_width)?;
        map.serialize_entry(
            "interlocking_boundary_avoidance",
            &self.interlocking_boundary_avoidance,
        )?;
        map.serialize_entry("interlocking_depth", &self.interlocking_depth)?;
        map.serialize_entry("interlocking_orientation", &self.interlocking_orientation)?;
        map.serialize_entry("internal_bridge_density", &self.internal_bridge_density)?;
        map.serialize_entry(
            "internal_solid_infill_acceleration",
            &self.internal_solid_infill_acceleration,
        )?;
        map.serialize_entry("layer_height", &self.layer_height)?;
        map.serialize_entry("line_width", &self.line_width)?;
        map.serialize_entry(
            "make_overhang_printable_angle",
            &self.make_overhang_printable_angle,
        )?;
        map.serialize_entry(
            "make_overhang_printable_hole_size",
            &self.make_overhang_printable_hole_size,
        )?;
        map.serialize_entry("max_bridge_length", &self.max_bridge_length)?;
        map.serialize_entry("min_bead_width", &self.min_bead_width)?;
        map.serialize_entry("min_feature_size", &self.min_feature_size)?;
        map.serialize_entry("min_length_factor", &self.min_length_factor)?;
        map.serialize_entry(
            "mmu_segmented_region_interlocking_depth",
            &self.mmu_segmented_region_interlocking_depth,
        )?;
        map.serialize_entry(
            "mmu_segmented_region_max_width",
            &self.mmu_segmented_region_max_width,
        )?;
        map.serialize_entry("outer_wall_acceleration", &self.outer_wall_acceleration)?;
        map.serialize_entry("outer_wall_jerk", &self.outer_wall_jerk)?;
        map.serialize_entry("precise_z_height", &self.precise_z_height)?;
        map.serialize_entry("raft_contact_distance", &self.raft_contact_distance)?;
        map.serialize_entry("raft_expansion", &self.raft_expansion)?;
        map.serialize_entry("raft_first_layer_density", &self.raft_first_layer_density)?;
        map.serialize_entry(
            "raft_first_layer_expansion",
            &self.raft_first_layer_expansion,
        )?;
        map.serialize_entry("raft_layers", &self.raft_layers)?;
        map.serialize_entry("seam_position", &self.seam_position)?;
        map.serialize_entry("set_other_flow_ratios", &self.set_other_flow_ratios)?;
        map.serialize_entry("skirt_start_angle", &self.skirt_start_angle)?;
        map.serialize_entry("slice_closing_radius", &self.slice_closing_radius)?;
        map.serialize_entry("slicing_mode", &self.slicing_mode)?;
        map.serialize_entry(
            "sparse_infill_acceleration",
            &self.sparse_infill_acceleration,
        )?;
        map.serialize_entry("staggered_inner_seams", &self.staggered_inner_seams)?;
        map.serialize_entry("support_angle", &self.support_angle)?;
        map.serialize_entry("support_base_pattern", &self.support_base_pattern)?;
        map.serialize_entry(
            "support_base_pattern_spacing",
            &self.support_base_pattern_spacing,
        )?;
        map.serialize_entry(
            "support_bottom_interface_spacing",
            &self.support_bottom_interface_spacing,
        )?;
        map.serialize_entry("support_bottom_z_distance", &self.support_bottom_z_distance)?;
        map.serialize_entry(
            "support_critical_regions_only",
            &self.support_critical_regions_only,
        )?;
        map.serialize_entry("support_expansion", &self.support_expansion)?;
        map.serialize_entry("support_filament", &self.support_filament)?;
        map.serialize_entry("support_flow_ratio", &self.support_flow_ratio)?;
        map.serialize_entry(
            "support_interface_bottom_layers",
            &self.support_interface_bottom_layers,
        )?;
        map.serialize_entry(
            "support_interface_filament",
            &self.support_interface_filament,
        )?;
        map.serialize_entry(
            "support_interface_flow_ratio",
            &self.support_interface_flow_ratio,
        )?;
        map.serialize_entry(
            "support_interface_loop_pattern",
            &self.support_interface_loop_pattern,
        )?;
        map.serialize_entry(
            "support_interface_not_for_body",
            &self.support_interface_not_for_body,
        )?;
        map.serialize_entry("support_interface_pattern", &self.support_interface_pattern)?;
        map.serialize_entry("support_interface_spacing", &self.support_interface_spacing)?;
        map.serialize_entry("support_interface_speed", &self.support_interface_speed)?;
        map.serialize_entry(
            "support_interface_top_layers",
            &self.support_interface_top_layers,
        )?;
        map.serialize_entry("support_ironing", &self.support_ironing)?;
        map.serialize_entry("support_ironing_flow", &self.support_ironing_flow)?;
        map.serialize_entry("support_ironing_pattern", &self.support_ironing_pattern)?;
        map.serialize_entry("support_ironing_spacing", &self.support_ironing_spacing)?;
        map.serialize_entry("support_line_width", &self.support_line_width)?;
        map.serialize_entry(
            "support_object_first_layer_gap",
            &self.support_object_first_layer_gap,
        )?;
        map.serialize_entry(
            "support_object_xy_distance",
            &self.support_object_xy_distance,
        )?;
        map.serialize_entry(
            "support_on_build_plate_only",
            &self.support_on_build_plate_only,
        )?;
        map.serialize_entry(
            "support_remove_small_overhang",
            &self.support_remove_small_overhang,
        )?;
        map.serialize_entry("support_speed", &self.support_speed)?;
        map.serialize_entry("support_style", &self.support_style)?;
        map.serialize_entry("support_threshold_angle", &self.support_threshold_angle)?;
        map.serialize_entry("support_threshold_overlap", &self.support_threshold_overlap)?;
        map.serialize_entry("support_top_z_distance", &self.support_top_z_distance)?;
        map.serialize_entry("support_type", &self.support_type)?;
        map.serialize_entry("thick_bridges", &self.thick_bridges)?;
        map.serialize_entry("thick_internal_bridges", &self.thick_internal_bridges)?;
        map.serialize_entry("top_surface_acceleration", &self.top_surface_acceleration)?;
        map.serialize_entry("top_surface_jerk", &self.top_surface_jerk)?;
        map.serialize_entry("travel_acceleration", &self.travel_acceleration)?;
        map.serialize_entry("travel_jerk", &self.travel_jerk)?;
        map.serialize_entry("tree_support_angle_slow", &self.tree_support_angle_slow)?;
        map.serialize_entry("tree_support_auto_brim", &self.tree_support_auto_brim)?;
        map.serialize_entry("tree_support_branch_angle", &self.tree_support_branch_angle)?;
        map.serialize_entry(
            "tree_support_branch_angle_organic",
            &self.tree_support_branch_angle_organic,
        )?;
        map.serialize_entry(
            "tree_support_branch_diameter",
            &self.tree_support_branch_diameter,
        )?;
        map.serialize_entry(
            "tree_support_branch_diameter_angle",
            &self.tree_support_branch_diameter_angle,
        )?;
        map.serialize_entry(
            "tree_support_branch_diameter_organic",
            &self.tree_support_branch_diameter_organic,
        )?;
        map.serialize_entry(
            "tree_support_branch_distance",
            &self.tree_support_branch_distance,
        )?;
        map.serialize_entry(
            "tree_support_branch_distance_organic",
            &self.tree_support_branch_distance_organic,
        )?;
        map.serialize_entry("tree_support_brim_width", &self.tree_support_brim_width)?;
        map.serialize_entry("tree_support_tip_diameter", &self.tree_support_tip_diameter)?;
        map.serialize_entry("tree_support_top_rate", &self.tree_support_top_rate)?;
        map.serialize_entry("tree_support_wall_count", &self.tree_support_wall_count)?;
        map.serialize_entry("wall_distribution_count", &self.wall_distribution_count)?;
        map.serialize_entry("wall_generator", &self.wall_generator)?;
        map.serialize_entry("wall_maximum_deviation", &self.wall_maximum_deviation)?;
        map.serialize_entry("wall_maximum_resolution", &self.wall_maximum_resolution)?;
        map.serialize_entry("wall_transition_angle", &self.wall_transition_angle)?;
        map.serialize_entry(
            "wall_transition_filter_deviation",
            &self.wall_transition_filter_deviation,
        )?;
        map.serialize_entry("wall_transition_length", &self.wall_transition_length)?;
        map.serialize_entry("xy_contour_compensation", &self.xy_contour_compensation)?;
        map.serialize_entry("xy_hole_compensation", &self.xy_hole_compensation)?;
        map.end()
    }
}
