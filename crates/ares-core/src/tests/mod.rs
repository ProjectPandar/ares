use super::*;
use serde_json::json;

#[test]
fn slice_options_preserve_unknown_orca_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "wall_loops": 2,
        "filament_colour": ["#FFFFFF"]
    }))
    .unwrap();

    assert_eq!(options.values().len(), 3);
    assert_eq!(options.values()["layer_height"], json!(0.2));
    assert_eq!(options.values()["wall_loops"], json!(2));
    assert_eq!(options.values()["filament_colour"], json!(["#FFFFFF"]));
}

mod acceleration_gcode;
mod adaptive_bed_mesh_gcode;
mod adaptive_bridge_pressure_advance_gcode;
mod aligned_rectilinear_infill_gcode;
mod auxiliary_fan_gcode;
mod bbl_bed_temperature_gcode;
mod bed_exclude_area_gcode;
mod bed_temperature_gcode;
mod bed_temperature_initial_layer_placeholder_gcode;
mod bed_temperature_initial_layer_vector_placeholder_gcode;
mod bed_temperature_placeholder_gcode;
mod brim_gcode;
mod btt_thumbnail_header_gcode;
mod chamber_temperature_gcode;
mod chamber_temperature_placeholder_gcode;
mod change_filament_gcode;
mod current_hotend_placeholder_gcode;
mod custom_gcode;
mod custom_gcode_end;
mod custom_gcode_filament_role_change;
mod custom_gcode_role_change;
mod custom_gcode_start;
mod default_filament_colour_gcode;
mod disable_m73_gcode;
mod dont_filter_internal_bridges_gcode;
mod during_print_exhaust_fan_speed_num_placeholder_gcode;
mod enable_high_low_temp_mix_placeholder_gcode;
mod exhaust_fan_gcode;
mod extruder_colour_gcode;
mod filament_adhesiveness_category_gcode;
mod filament_change_gcode;
mod filament_change_length_gcode;
mod filament_colour_gcode;
mod filament_colour_new_gcode;
mod filament_colour_type_gcode;
mod filament_cooling_before_tower_gcode;
mod filament_cooling_final_speed_gcode;
mod filament_cooling_initial_speed_gcode;
mod filament_cooling_moves_gcode;
mod filament_ids_gcode;
mod filament_is_support_gcode;
mod filament_load_unload_speed_gcode;
mod filament_map_gcode;
mod filament_minimal_purge_gcode;
mod filament_multi_colour_gcode;
mod filament_multitool_ramming_gcode;
mod filament_printable_gcode;
mod filament_ramming_parameters_gcode;
mod filament_shrink_xy_gcode;
mod filament_soluble_gcode;
mod filament_stamping_gcode;
mod filament_toolchange_delay_gcode;
mod filament_tower_interface_pre_extrusion_dist_gcode;
mod filament_tower_interface_pre_extrusion_length_gcode;
mod filament_tower_interface_print_temp_gcode;
mod filament_tower_interface_purge_volume_gcode;
mod filament_tower_ironing_area_gcode;
mod filament_type_gcode;
mod filament_z_shrinkage_gcode;
mod first_layer_height_placeholder_gcode;
mod first_layer_print_placeholders_gcode;
mod first_tools_placeholders_gcode;
mod flush_placeholders_gcode;
mod fuzzy_skin_gcode;
mod gcode_add_line_number;
mod gcode_comments;
mod gcode_flavor_gcode;
mod gcode_label_objects;
mod has_tpu_in_first_layer_gcode;
mod head_wrap_detect_zone_placeholder_gcode;
mod idle_standby_startup_temperature_gcode;
mod initial_extruder_placeholders_gcode;
mod initial_layer_print_height_gcode;
mod initial_layer_travel_acceleration_gcode;
mod input_shaping_gcode;
mod internal_solid_infill_gcode;
mod is_all_bbl_filament_gcode;
mod is_extruder_used_placeholder_gcode;
mod jerk_gcode;
mod layer_change_retraction_gcode;
mod layer_gcode;
mod long_retraction_when_cut_placeholder_gcode;
mod long_retraction_when_ec_placeholder_gcode;
mod long_retractions_when_cut_vector_placeholder_gcode;
mod long_retractions_when_ec_vector_placeholder_gcode;
mod machine_limits_gcode;
mod machine_min_rate_time_gcode;
mod machine_start_stat_reserved_placeholders_gcode;
mod max_print_height_placeholder_gcode;
mod max_print_z_placeholder_gcode;
mod min_skirt_length_gcode;
mod mmu_scalar_config_header_gcode;
mod model_import;
mod non_support_tool_placeholders_gcode;
mod nozzle_temperature_gcode;
mod num_extruders_gcode;
mod other_layer_temperature_gcode;
mod outer_wall_volumetric_speed_placeholder_gcode;
mod part_cooling_fan_gcode;
mod pellet_flow_gcode;
mod physical_extruder_map_gcode;
mod power_loss_recovery_gcode;
mod preheat_gcode;
mod pressure_advance_gcode;
mod prime_tower_brim_width_header_gcode;
mod prime_tower_width_header_gcode;
mod print_bed_placeholders_gcode;
mod print_sequence_gcode;
mod relative_e_gcode;
mod required_nozzle_hrc_gcode;
mod retract_length_placeholder_gcode;
mod retraction_distance_when_cut_placeholder_gcode;
mod retraction_distance_when_ec_placeholder_gcode;
mod retraction_distances_when_cut_vector_placeholder_gcode;
mod retraction_distances_when_ec_vector_placeholder_gcode;
mod scan_first_layer_gcode;
mod single_loop_draft_shield_gcode;
mod skirt_gcode;
mod skirt_start_angle_gcode;
mod skirt_type_gcode;
mod small_area_flow_model_header_gcode;
mod speed_gcode;
mod spiral_mode_normalization_gcode;
mod temperature_vitrification_gcode;
mod timelapse_type_gcode;
mod total_layer_count_gcode;
mod travel_acceleration_gcode;
mod travel_retraction_gcode;
mod wipe_tower_config_header_gcode;
mod wipe_tower_coordinate_header_gcode;
mod wipe_tower_placeholders_gcode;
mod wrapping_detection_gcode;
mod z_offset_gcode;
mod z_offset_placeholder_gcode;

#[tokio::test]
async fn slice_returns_deterministic_gcode_for_stl() {
    let output = slice(square_pyramid_ascii_stl(), SliceOptions::default())
        .await
        .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; generated by Ares"));
    assert!(output.contains("; input_format = stl"));
    assert!(output.contains("; triangle_count = 4"));
    assert!(output.contains("; layer_count = 2"));
    assert!(output.contains("; option_count = 0"));
    assert!(output.ends_with("M2\n"));
}

#[tokio::test]
async fn slice_gcode_bytes_remain_unchanged_after_print_domain_view() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();

    assert_eq!(output.len(), 4753);
    assert_eq!(fnv1a64(&output), 0x8990a54281eb9dfd);
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[tokio::test]
async fn slice_rejects_3mf_until_geometry_extraction_exists() {
    let err = slice(b"PK\x03\x04fake-3mf", SliceOptions::default())
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

fn square_pyramid_ascii_stl() -> Vec<u8> {
    [
        "solid pyramid",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 0.4",
        "vertex 0 1 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 -1 0.4",
        "vertex 1 0 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex -1 0 0.4",
        "vertex 0 -1 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 1 0.4",
        "vertex -1 0 0.4",
        "endloop",
        "endfacet",
        "endsolid pyramid",
    ]
    .join("\n")
    .into_bytes()
}

#[tokio::test]
async fn slice_emits_hardware_option_metadata() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": ["0.4", "0.6"],
        "filament_diameter": "1.75;2.85",
        "min_layer_height": "0.06,0.08",
        "max_layer_height": [0.28, 0.42]
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; nozzle_diameter = 0.4,0.6"));
    assert!(output.contains("; filament_diameter = 1.75,2.85"));
    assert!(output.contains("; min_layer_height = 0.06,0.08"));
    assert!(output.contains("; max_layer_height = 0.28,0.42"));
}

#[tokio::test]
async fn slice_accepts_merged_profile_options() {
    let fragments = [
        ProfileFragment::from_json_bytes(
            br#"{"type":"process","name":"base","layer_height":0.2,"initial_layer_height":0.2,"nozzle_diameter":["0.4"],"filament_diameter":["1.75"]}"#,
        )
        .unwrap(),
        ProfileFragment::from_json_bytes(
            br#"{"type":"process","name":"fine","inherits":"base","layer_height":0.1,"min_layer_height":["0.05"],"max_layer_height":["0.24"]}"#,
        )
        .unwrap(),
    ];
    let options = merge_profile_fragments(&fragments, ProfileKind::Process, "fine").unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; layer_height = 0.1"));
    assert!(output.contains("; initial_layer_height = 0.2"));
    assert!(output.contains("; nozzle_diameter = 0.4"));
    assert!(output.contains("; filament_diameter = 1.75"));
    assert!(output.contains("; min_layer_height = 0.05"));
    assert!(output.contains("; max_layer_height = 0.24"));
}

#[tokio::test]
async fn slice_accepts_composed_profile_options() {
    let fragments = [
        ProfileFragment::from_json_bytes(
            br#"{"type":"machine","name":"printer","nozzle_diameter":["0.6"],"min_layer_height":["0.08"],"max_layer_height":["0.32"]}"#,
        )
        .unwrap(),
        ProfileFragment::from_json_bytes(
            br#"{"type":"process","name":"fine","layer_height":0.1,"initial_layer_height":0.2}"#,
        )
        .unwrap(),
        ProfileFragment::from_json_bytes(
            br#"{"type":"filament","name":"pla","filament_diameter":["2.85"]}"#,
        )
        .unwrap(),
    ];
    let selection = ProfileSelection::new("fine", "printer", ["pla"]).unwrap();
    let options = compose_profile_fragments(&fragments, &selection)
        .unwrap()
        .into_options();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.lines().any(|line| line == "; layer_height = 0.1"));
    assert!(
        output
            .lines()
            .any(|line| line == "; initial_layer_height = 0.2")
    );
    assert!(output.lines().any(|line| line == "; nozzle_diameter = 0.6"));
    assert!(
        output
            .lines()
            .any(|line| line == "; filament_diameter = 2.85")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; min_layer_height = 0.08")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; max_layer_height = 0.32")
    );
}

#[tokio::test]
async fn slice_rejects_empty_input() {
    let err = slice([], SliceOptions::default()).await.unwrap_err();

    assert!(matches!(err, SliceError::EmptyInput));
}
