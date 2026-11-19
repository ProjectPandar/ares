use ares_core::{option_definition, option_metadata};

#[test]
fn upstream_nullable_options_are_nullable_in_public_registry() {
    // PrintConfig.cpp definition flags and the cloned nullable support speed default.
    const UPSTREAM_NULLABLE: &[&str] = &[
        "bridge_acceleration",
        "bridge_speed",
        "default_acceleration",
        "default_jerk",
        "default_junction_deviation",
        "deretract_speed_extruder_change",
        "enable_overhang_speed",
        "extruder_max_nozzle_count",
        "filament_adaptive_volumetric_speed",
        "filament_cooling_before_tower",
        "filament_flow_ratio",
        "filament_flush_temp",
        "filament_flush_temp_fast",
        "filament_flush_volumetric_speed",
        "filament_ironing_flow",
        "filament_ironing_inset",
        "filament_ironing_spacing",
        "filament_ironing_speed",
        "filament_pre_cooling_temperature",
        "filament_pre_cooling_temperature_nc",
        "filament_preheat_temperature_delta",
        "filament_ramming_travel_time",
        "filament_ramming_travel_time_nc",
        "filament_ramming_volumetric_speed",
        "filament_ramming_volumetric_speed_nc",
        "filament_retract_length_nc",
        "gap_infill_speed",
        "hotend_cooling_rate",
        "hotend_heating_rate",
        "infill_jerk",
        "initial_layer_acceleration",
        "initial_layer_infill_speed",
        "initial_layer_jerk",
        "initial_layer_speed",
        "initial_layer_travel_acceleration",
        "initial_layer_travel_jerk",
        "initial_layer_travel_speed",
        "inner_wall_acceleration",
        "inner_wall_jerk",
        "inner_wall_speed",
        "internal_bridge_speed",
        "internal_solid_infill_acceleration",
        "internal_solid_infill_speed",
        "long_retractions_when_ec",
        "nozzle_flush_dataset",
        "nozzle_type",
        "nozzle_volume",
        "outer_wall_acceleration",
        "outer_wall_jerk",
        "outer_wall_speed",
        "overhang_1_4_speed",
        "overhang_2_4_speed",
        "overhang_3_4_speed",
        "overhang_4_4_speed",
        "retraction_distances_when_ec",
        "slowdown_for_curled_perimeters",
        "small_perimeter_speed",
        "small_perimeter_threshold",
        "small_support_perimeter_speed",
        "sparse_infill_acceleration",
        "sparse_infill_speed",
        "support_interface_speed",
        "support_speed",
        "top_solid_infill_flow_ratio",
        "top_surface_acceleration",
        "top_surface_jerk",
        "top_surface_speed",
        "travel_acceleration",
        "travel_jerk",
        "travel_speed",
        "travel_speed_z",
    ];

    for &key in UPSTREAM_NULLABLE {
        assert!(
            option_definition(key).is_some(),
            "missing option definition for {key}"
        );
        let metadata =
            option_metadata(key).unwrap_or_else(|| panic!("missing option metadata for {key}"));
        assert!(
            metadata.nullable,
            "option metadata is not nullable for {key}"
        );
    }
}
