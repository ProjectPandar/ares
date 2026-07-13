use std::collections::BTreeSet;

use super::super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, OrcaInts, OrcaString, OrcaStrings, Percent,
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessInfillPattern, ProcessIroningType, ProcessNoiseType,
    ProcessRegionSourceOptions, ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
    RegionOptions,
};

pub(super) fn assert_concrete_types_and_identity(
    effective: &RegionOptions,
    source: &ProcessRegionSourceOptions,
) {
    let mut count = 0;
    let mut names = BTreeSet::new();
    fields!(effective, source, count, names, OrcaBool;
        relative_bridge_angle, symmetric_infill_y_axis, align_infill_direction_to_model,
        fuzzy_skin_first_layer, infill_combination, gyroid_optimized, ironing_angle_fixed,
        detect_overhang_wall, alternate_extra_wall, detect_thin_wall, enable_overhang_speed,
        only_one_wall_top, only_one_wall_first_layer, role_based_wipe_speed, wipe_on_loops,
        wipe_before_external_loop, precise_outer_wall, make_overhang_printable,
        extra_perimeters_on_overhangs, slowdown_for_curled_perimeters, hole_to_polyhole,
        hole_to_polyhole_twisted, overhang_reverse, overhang_reverse_internal_only,
        is_infill_first, small_area_infill_flow_compensation, seam_slope_conditional,
        seam_slope_entire_loop, seam_slope_inner_walls, zaa_enabled,
        zaa_dont_alternate_fill_direction
    );
    fields!(effective, source, count, names, OrcaFloat;
        bottom_shell_thickness, bridge_angle, internal_bridge_angle, bridge_flow,
        internal_bridge_flow, bridge_speed, outer_wall_speed, infill_direction,
        solid_infill_direction, infill_shift_step, lateral_lattice_angle_1,
        lateral_lattice_angle_2, infill_overhang_angle, lightning_overhang_angle,
        lightning_prune_angle, lightning_straightening_angle, fuzzy_skin_thickness,
        fuzzy_skin_point_distance, fuzzy_skin_scale, fuzzy_skin_persistence, gap_infill_speed,
        sparse_infill_speed, infill_lock_depth, skin_infill_depth, ironing_spacing,
        ironing_inset, ironing_speed, ironing_angle, inner_wall_speed,
        minimum_sparse_infill_area, internal_solid_infill_speed, top_shell_thickness,
        top_surface_speed, print_flow_ratio, filter_out_gap_fill, small_perimeter_threshold,
        top_solid_infill_flow_ratio, bottom_solid_infill_flow_ratio, first_layer_flow_ratio,
        outer_wall_flow_ratio, inner_wall_flow_ratio, overhang_flow_ratio,
        sparse_infill_flow_ratio, internal_solid_infill_flow_ratio, gap_fill_flow_ratio,
        seam_slope_min_length, scarf_joint_flow_ratio, zaa_min_z, zaa_minimize_perimeter_height
    );
    fields!(effective, source, count, names, FloatOrPercent;
        bridge_line_width, internal_bridge_speed, outer_wall_line_width,
        sparse_infill_line_width, skin_infill_line_width, skeleton_infill_line_width,
        infill_combination_max_layer_height, inner_wall_line_width,
        internal_solid_infill_line_width, top_surface_line_width, overhang_1_4_speed,
        overhang_2_4_speed, overhang_3_4_speed, overhang_4_4_speed, min_width_top_surface,
        seam_gap, wipe_speed, small_perimeter_speed, infill_anchor, infill_anchor_max,
        hole_to_polyhole_threshold, overhang_reverse_threshold, seam_slope_start_height,
        scarf_joint_speed
    );
    fields!(effective, source, count, names, OrcaInt;
        bottom_shell_layers, fuzzy_skin_octaves, fuzzy_skin_ripples_per_layer,
        fuzzy_skin_layers_between_ripple_offset, sparse_infill_filament_id, fill_multiline,
        outer_wall_filament_id, inner_wall_filament_id, wall_loops,
        internal_solid_filament_id, top_surface_filament_id, bottom_surface_filament_id,
        top_shell_layers, scarf_angle_threshold, seam_slope_steps
    );
    fields!(effective, source, count, names, Percent;
        top_surface_density, bottom_surface_density, sparse_infill_density,
        fuzzy_skin_ripple_offset, infill_wall_overlap, top_bottom_infill_wall_overlap,
        skeleton_infill_density, skin_infill_density, ironing_flow, bridge_density,
        scarf_overhang_threshold
    );
    fields!(effective, source, count, names, OrcaInts; print_extruder_id);
    fields!(effective, source, count, names, OrcaString;
        solid_infill_rotate_template, sparse_infill_rotate_template, extra_solid_infills
    );
    fields!(effective, source, count, names, OrcaStrings; print_extruder_variant);
    fields!(effective, source, count, names, ProcessInfillPattern;
        top_surface_pattern, bottom_surface_pattern, internal_solid_infill_pattern,
        sparse_infill_pattern, ironing_pattern
    );
    fields!(effective, source, count, names, ProcessEnsureVerticalShellThickness;
        ensure_vertical_shell_thickness
    );
    fields!(effective, source, count, names, ProcessFuzzySkinType; fuzzy_skin);
    fields!(effective, source, count, names, ProcessNoiseType; fuzzy_skin_noise_type);
    fields!(effective, source, count, names, ProcessFuzzySkinMode; fuzzy_skin_mode);
    fields!(effective, source, count, names, ProcessIroningType; ironing_type);
    fields!(effective, source, count, names, ProcessCounterboreHoleBridging;
        counterbore_hole_bridging
    );
    fields!(effective, source, count, names, ProcessWallSequence; wall_sequence);
    fields!(effective, source, count, names, ProcessWallDirection; wall_direction);
    fields!(effective, source, count, names, ProcessSeamScarfType; seam_slope_type);

    let _: &Percent = &effective.filament_ironing_flow;
    let _: &OrcaFloat = &effective.filament_ironing_spacing;
    let _: &OrcaFloat = &effective.filament_ironing_inset;
    let _: &OrcaFloat = &effective.filament_ironing_speed;
    assert_eq!(count, 149);
    assert_eq!(
        names,
        RegionOptions::PROCESS_DECLARATION_ORDER
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
}

macro_rules! fields {
    ($effective:ident, $source:ident, $count:ident, $names:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(
            let _: &$ty = &$effective.$field;
            let _: &$ty = &$source.$field;
            assert_eq!(&$effective.$field, &$source.$field, stringify!($field));
            $count += 1;
            assert!($names.insert(stringify!($field)), stringify!($field));
        )+
    };
}

use fields;
