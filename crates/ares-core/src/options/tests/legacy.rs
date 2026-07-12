use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_supertack_bed_type_spelling() {
    let options: SliceOptions = serde_json::from_value(json!({
        "curr_bed_type": "SuperTack Plate"
    }))
    .unwrap();

    assert_eq!(options.values()["curr_bed_type"], json!("Supertack Plate"));
}

#[test]
fn normalizes_simple_legacy_option_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "enable_wipe_tower": true,
        "wipe_tower_width": 64,
        "wiping_volume": 123,
        "wipe_tower_brim_width": 4,
        "tool_change_gcode": "M600",
        "bridge_fan_speed": [80],
        "infill_extruder": 2,
        "solid_infill_extruder": 3,
        "perimeter_extruder": 4,
        "wipe_tower_extruder": 5,
        "support_material_extruder": 6,
        "support_material_interface_extruder": 7,
        "support_material_angle": 33,
        "support_material_enforce_layers": 2,
        "future_orca_key": "preserved"
    }))
    .unwrap();

    for legacy_key in [
        "enable_wipe_tower",
        "wipe_tower_width",
        "wiping_volume",
        "wipe_tower_brim_width",
        "tool_change_gcode",
        "bridge_fan_speed",
        "infill_extruder",
        "solid_infill_extruder",
        "perimeter_extruder",
        "wipe_tower_extruder",
        "support_material_extruder",
        "support_material_interface_extruder",
        "support_material_angle",
        "support_material_enforce_layers",
    ] {
        assert!(!options.values().contains_key(legacy_key));
    }
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(options.values()["prime_tower_width"], json!(64));
    assert_eq!(options.values()["prime_volume"], json!(123));
    assert_eq!(options.values()["prime_tower_brim_width"], json!(4));
    assert_eq!(options.values()["change_filament_gcode"], json!("M600"));
    assert_eq!(options.values()["overhang_fan_speed"], json!([80]));
    assert_eq!(options.values()["sparse_infill_filament"], json!(2));
    assert_eq!(options.values()["solid_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_filament"], json!(4));
    assert_eq!(options.values()["wipe_tower_filament"], json!(5));
    assert_eq!(options.values()["support_filament"], json!(6));
    assert_eq!(options.values()["support_interface_filament"], json!(7));
    assert_eq!(options.values()["support_angle"], json!(33));
    assert_eq!(options.values()["enforce_support_layers"], json!(2));
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn drops_legacy_percentage_values_for_now_absolute_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "initial_layer_print_height": "50%",
        "initial_layer_speed": "30%",
        "internal_solid_infill_speed": "40%",
        "top_surface_speed": "45%",
        "support_interface_speed": "60%",
        "outer_wall_speed": "70%",
        "support_object_xy_distance": "80%",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    for key in [
        "initial_layer_print_height",
        "initial_layer_speed",
        "internal_solid_infill_speed",
        "top_surface_speed",
        "outer_wall_speed",
        "support_object_xy_distance",
    ] {
        assert!(!options.values().contains_key(key));
    }
    assert_eq!(options.values()["support_interface_speed"], json!("60%"));
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn preserves_now_absolute_options_when_values_are_not_percentage_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "initial_layer_print_height": "0.2",
        "initial_layer_speed": 30,
        "internal_solid_infill_speed": "40mm/s",
        "top_surface_speed": "45",
        "support_interface_speed": 60,
        "outer_wall_speed": "70",
        "support_object_xy_distance": 0.35
    }))
    .unwrap();

    assert_eq!(options.values()["initial_layer_print_height"], json!("0.2"));
    assert_eq!(options.values()["initial_layer_speed"], json!(30));
    assert_eq!(
        options.values()["internal_solid_infill_speed"],
        json!("40mm/s")
    );
    assert_eq!(options.values()["top_surface_speed"], json!("45"));
    assert_eq!(options.values()["support_interface_speed"], json!(60));
    assert_eq!(options.values()["outer_wall_speed"], json!("70"));
    assert_eq!(options.values()["support_object_xy_distance"], json!(0.35));
}

#[test]
fn normalizes_legacy_cumulative_cooling_and_timelapse_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "inherits_cummulative": ["base"],
        "compatible_printers_condition_cummulative": ["printer"],
        "compatible_prints_condition_cummulative": ["process"],
        "cooling": [true],
        "timelapse_no_toolhead": "2"
    }))
    .unwrap();

    for legacy_key in [
        "inherits_cummulative",
        "compatible_printers_condition_cummulative",
        "compatible_prints_condition_cummulative",
        "cooling",
        "timelapse_no_toolhead",
    ] {
        assert!(!options.values().contains_key(legacy_key));
    }
    assert_eq!(options.values()["inherits_group"], json!(["base"]));
    assert_eq!(
        options.values()["compatible_machine_expression_group"],
        json!(["printer"])
    );
    assert_eq!(
        options.values()["compatible_process_expression_group"],
        json!(["process"])
    );
    assert_eq!(
        options.values()["slow_down_for_layer_cooling"],
        json!([true])
    );
    assert_eq!(options.values()["timelapse_type"], json!("2"));
}

#[test]
fn normalizes_legacy_timelapse_and_support_values() {
    let migrated: SliceOptions = serde_json::from_value(json!({
        "timelapse_type": "2",
        "support_type": "normal",
        "support_base_pattern": "none"
    }))
    .unwrap();
    assert_eq!(migrated.values()["timelapse_type"], json!("0"));
    assert_eq!(migrated.values()["support_type"], json!("normal(manual)"));
    assert_eq!(migrated.values()["support_base_pattern"], json!("hollow"));

    let tree: SliceOptions = serde_json::from_value(json!({"support_type": "tree"})).unwrap();
    assert_eq!(tree.values()["support_type"], json!("tree(manual)"));

    let hybrid: SliceOptions =
        serde_json::from_value(json!({"support_type": "hybrid(auto)"})).unwrap();
    assert_eq!(hybrid.values()["support_type"], json!("tree(auto)"));

    let unchanged: SliceOptions = serde_json::from_value(json!({
        "timelapse_type": "1",
        "support_type": "normal(auto)",
        "support_base_pattern": "grid"
    }))
    .unwrap();
    assert_eq!(unchanged.values()["timelapse_type"], json!("1"));
    assert_eq!(unchanged.values()["support_type"], json!("normal(auto)"));
    assert_eq!(unchanged.values()["support_base_pattern"], json!("grid"));
}

#[test]
fn normalizes_legacy_keys_inside_different_settings_to_system() {
    let options: SliceOptions = serde_json::from_value(json!({
        "different_settings_to_system": "\"enable_wipe_tower\";inherits_cummulative;support_material_angle;future_orca_key",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert_eq!(
        options.values()["different_settings_to_system"],
        json!("\"enable_prime_tower\";inherits_group;support_angle;future_orca_key")
    );
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn different_settings_to_system_deduplicates_and_skips_value_only_migrations() {
    let options: SliceOptions = serde_json::from_value(json!({
        "different_settings_to_system": "enable_wipe_tower;enable_wipe_tower;timelapse_type;support_type;support_base_pattern"
    }))
    .unwrap();

    assert_eq!(
        options.values()["different_settings_to_system"],
        json!(
            "enable_prime_tower;enable_prime_tower;timelapse_type;support_type;support_base_pattern"
        )
    );
}

#[test]
fn preserves_non_string_different_settings_to_system_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "different_settings_to_system": ["enable_wipe_tower"]
    }))
    .unwrap();

    assert_eq!(
        options.values()["different_settings_to_system"],
        json!(["enable_wipe_tower"])
    );
}

#[test]
fn normalizes_legacy_overhang_fan_threshold_value() {
    let migrated: SliceOptions = serde_json::from_value(json!({
        "overhang_fan_threshold": "5%",
        "future_orca_key": "preserved"
    }))
    .unwrap();
    assert_eq!(migrated.values()["overhang_fan_threshold"], json!("10%"));
    assert_eq!(migrated.values()["future_orca_key"], json!("preserved"));

    let unchanged: SliceOptions = serde_json::from_value(json!({
        "overhang_fan_threshold": "15%"
    }))
    .unwrap();
    assert_eq!(unchanged.values()["overhang_fan_threshold"], json!("15%"));
}

#[test]
fn normalizes_legacy_wall_infill_order_values() {
    for (legacy_value, expected) in [
        ("inner wall/outer wall/infill", "inner wall/outer wall"),
        ("infill/inner wall/outer wall", "inner wall/outer wall"),
        ("outer wall/inner wall/infill", "outer wall/inner wall"),
        ("infill/outer wall/inner wall", "outer wall/inner wall"),
        ("inner-outer-inner wall/infill", "inner-outer-inner wall"),
        ("custom order", "custom order"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "wall_infill_order": legacy_value
        }))
        .unwrap();

        assert!(!options.values().contains_key("wall_infill_order"));
        assert_eq!(options.values()["wall_sequence"], json!(expected));
    }
}

#[test]
fn normalizes_legacy_nozzle_and_extruder_variant_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_volume_type": "Normal;Big Traffic;Normal",
        "default_nozzle_volume_type": "Big Traffic",
        "printer_extruder_variant": "Normal",
        "print_extruder_variant": "Big Traffic",
        "filament_extruder_variant": "Normal Big Traffic",
        "extruder_variant_list": "Normal;Big Traffic",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert_eq!(
        options.values()["nozzle_volume_type"],
        json!("Standard;High Flow;Standard")
    );
    assert_eq!(
        options.values()["default_nozzle_volume_type"],
        json!("High Flow")
    );
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!("Standard")
    );
    assert_eq!(
        options.values()["print_extruder_variant"],
        json!("High Flow")
    );
    assert_eq!(
        options.values()["filament_extruder_variant"],
        json!("Standard High Flow")
    );
    assert_eq!(
        options.values()["extruder_variant_list"],
        json!("Standard;High Flow")
    );
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn normalizes_legacy_extruder_type_and_preserves_non_string_variant_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "extruder_type": "DirectDrive;DirectDrive",
        "nozzle_volume_type": ["Normal"],
        "printer_extruder_variant": 1
    }))
    .unwrap();

    assert_eq!(
        options.values()["extruder_type"],
        json!("Direct Drive;Direct Drive")
    );
    assert_eq!(options.values()["nozzle_volume_type"], json!(["Normal"]));
    assert_eq!(options.values()["printer_extruder_variant"], json!(1));
}
