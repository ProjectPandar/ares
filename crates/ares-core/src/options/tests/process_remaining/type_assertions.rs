use super::super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, OrcaString, OrcaStrings, Percent,
    ProcessDraftShield, ProcessGCodeSourceOptions, ProcessPrintOrder, ProcessPrintSequence,
    ProcessPrintSourceOptions, ProcessSkirtType, ProcessTimelapseType, ProcessWipeTowerWallType,
};

pub(super) fn assert_gcode_types(value: &ProcessGCodeSourceOptions) {
    fields!(value, OrcaBool;
        enable_arc_fitting, enable_wrapping_detection, gcode_add_line_number,
        extrusion_rate_smoothing_external_perimeter_only,
        single_extruder_multi_material_priming, wipe_tower_no_sparse_layers,
        accel_to_decel_enable
    );
    fields!(value, OrcaFloat;
        max_volumetric_extrusion_rate_slope,
        max_volumetric_extrusion_rate_slope_segment_length, travel_speed, travel_speed_z
    );
    fields!(value, FloatOrPercent;
        initial_layer_travel_speed, initial_layer_travel_acceleration,
        initial_layer_travel_jerk
    );
    let _: &Percent = &value.accel_to_decel_factor;
    let _: &OrcaString = &value.process_change_extrusion_role_gcode;
    let _: &OrcaStrings = &value.small_area_infill_flow_compensation_model;
}

pub(super) fn assert_print_types(value: &ProcessPrintSourceOptions) {
    fields!(value, OrcaBool;
        reduce_crossing_wall, reduce_infill_retraction, ooze_prevention, single_loop_draft_shield,
        spiral_mode, spiral_mode_smooth, enable_prime_tower, prime_tower_enable_framework,
        prime_tower_skip_points, prime_tower_flat_ironing, enable_tower_interface_features,
        enable_tower_interface_cooldown_during_tower, wipe_tower_fillet_wall,
        independent_support_layer_height, combine_brims, gcode_label_objects, exclude_object,
        gcode_comments
    );
    fields!(value, OrcaFloat;
        initial_layer_print_height, initial_layer_speed, initial_layer_infill_speed, resolution,
        skirt_distance, skirt_speed, min_skirt_length, spiral_finishing_flow_ratio,
        spiral_starting_flow_ratio, preheat_time, prime_tower_width, wipe_tower_rotation_angle,
        prime_tower_brim_width, wipe_tower_bridging, wipe_tower_cone_angle,
        wipe_tower_max_purge_speed, wipe_tower_extra_rib_length, wipe_tower_rib_width,
        prime_volume
    );
    fields!(value, FloatOrPercent;
        max_travel_detour_distance, initial_layer_line_width, spiral_mode_max_xy_smoothing
    );
    fields!(value, OrcaInt;
        skirt_height, skirt_loops, standby_temperature_delta, preheat_steps,
        wipe_tower_filament, slow_down_layers
    );
    fields!(value, Percent;
        prime_tower_infill_gap, wipe_tower_extra_flow, wipe_tower_extra_spacing
    );
    let _: &ProcessPrintSequence = &value.print_sequence;
    let _: &ProcessPrintOrder = &value.print_order;
    let _: &ProcessDraftShield = &value.draft_shield;
    let _: &ProcessSkirtType = &value.skirt_type;
    let _: &ProcessWipeTowerWallType = &value.wipe_tower_wall_type;
    let _: &ProcessTimelapseType = &value.timelapse_type;
    let _: &OrcaString = &value.filename_format;
    let _: &OrcaString = &value.notes;
    let _: &OrcaStrings = &value.post_process;
    let _: &OrcaFloats = &value.wiping_volumes_extruders;
}

macro_rules! fields {
    ($value:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(let _: &$ty = &$value.$field;)+
    };
}

use fields;
