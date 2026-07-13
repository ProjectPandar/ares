use super::super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessBrimType,
    ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessObjectSourceOptions, ProcessPerimeterGenerator,
    ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
};

pub fn assert_concrete_types(value: &ProcessObjectSourceOptions) {
    fields!(value, OrcaBool;
        brim_use_efc_outline, bridge_no_support, interface_shells, staggered_inner_seams,
        enable_support, support_on_build_plate_only, support_critical_regions_only,
        support_remove_small_overhang, support_interface_not_for_body,
        support_interface_loop_pattern, set_other_flow_ratios, thick_bridges,
        thick_internal_bridges, support_ironing, flush_into_objects, flush_into_infill,
        flush_into_support, tree_support_auto_brim, detect_narrow_internal_solid_infill,
        precise_z_height, interlocking_beam, calib_flowrate_topinfill_special_order
    );
    fields!(value, OrcaFloat;
        brim_object_gap, brim_flow_ratio, brim_width, brim_ears_detection_length,
        brim_ears_max_angle, skirt_start_angle, elefant_foot_compensation, max_bridge_length,
        layer_height, mmu_segmented_region_max_width, mmu_segmented_region_interlocking_depth,
        raft_contact_distance, raft_expansion, raft_first_layer_expansion, slice_closing_radius,
        support_angle, support_top_z_distance, support_bottom_z_distance,
        support_interface_spacing, support_interface_speed, support_base_pattern_spacing,
        support_expansion, support_speed, support_flow_ratio, support_interface_flow_ratio,
        support_object_xy_distance, support_object_first_layer_gap, support_ironing_spacing,
        xy_hole_compensation, xy_contour_compensation, tree_support_branch_distance,
        tree_support_tip_diameter, tree_support_branch_diameter, tree_support_branch_angle,
        tree_support_branch_diameter_angle, tree_support_angle_slow, tree_support_brim_width,
        support_bottom_interface_spacing, wall_transition_angle, wall_maximum_resolution,
        wall_maximum_deviation, make_overhang_printable_angle,
        make_overhang_printable_hole_size, tree_support_branch_distance_organic,
        tree_support_branch_diameter_organic, tree_support_branch_angle_organic,
        min_length_factor, default_acceleration, outer_wall_acceleration,
        inner_wall_acceleration, top_surface_acceleration, initial_layer_acceleration,
        travel_acceleration, default_jerk, outer_wall_jerk, inner_wall_jerk, infill_jerk,
        top_surface_jerk, initial_layer_jerk, travel_jerk, default_junction_deviation,
        interlocking_beam_width, interlocking_orientation
    );
    fields!(value, OrcaInt;
        elefant_foot_compensation_layers, raft_layers, enforce_support_layers, support_filament,
        support_interface_filament, support_interface_top_layers,
        support_interface_bottom_layers, support_threshold_angle, tree_support_wall_count,
        wall_distribution_count, interlocking_beam_layer_count, interlocking_depth,
        interlocking_boundary_avoidance
    );
    fields!(value, Percent;
        elefant_foot_layers_density, raft_first_layer_density, internal_bridge_density,
        support_ironing_flow, wall_transition_length, wall_transition_filter_deviation,
        min_feature_size, initial_layer_min_bead_width, min_bead_width, tree_support_top_rate
    );
    fields!(value, FloatOrPercent;
        line_width, support_line_width, support_threshold_overlap, bridge_acceleration,
        sparse_infill_acceleration, internal_solid_infill_acceleration
    );
    let _: &ProcessBrimType = &value.brim_type;
    let _: &ProcessSeamPosition = &value.seam_position;
    let _: &ProcessSlicingMode = &value.slicing_mode;
    let _: &ProcessSupportType = &value.support_type;
    let _: &ProcessSupportBasePattern = &value.support_base_pattern;
    let _: &ProcessSupportInterfacePattern = &value.support_interface_pattern;
    let _: &ProcessSupportStyle = &value.support_style;
    let _: &ProcessInternalBridgeFilter = &value.dont_filter_internal_bridges;
    let _: &ProcessExtraBridgeLayer = &value.enable_extra_bridge_layer;
    let _: &ProcessInfillPattern = &value.support_ironing_pattern;
    let _: &ProcessPerimeterGenerator = &value.wall_generator;
    let _: &ProcessGapFillTarget = &value.gap_fill_target;
}

macro_rules! fields {
    ($value:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(let _: &$ty = &$value.$field;)+
    };
}

use fields;
