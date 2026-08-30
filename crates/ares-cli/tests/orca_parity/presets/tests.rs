use super::VendorProfiles;
use crate::{
    normalize_process_defaults, runner, select_printer, smoke_case_overrides, smoke_overrides,
};

fn profiles(vendor: &str) -> VendorProfiles {
    VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        vendor,
    )
    .unwrap()
}

#[test]
fn stale_vendor_custom_gcode_placeholders_are_normalized() {
    let machine = serde_json::from_value(serde_json::json!({
        "machine_start_gcode": "M117 [output_filename_format]\nSTART V={extruder_rotation_volume[0]} Z={multi_zone_1_initial_layer[0]}"
    }))
    .unwrap();
    let process = serde_json::Map::new();

    let overrides = smoke_case_overrides(&machine, &process);
    let start = overrides["machine_start_gcode"].as_str().unwrap();

    assert!(start.contains("M117 [input_filename_base]"));
    assert!(start.contains("START V=0 Z=0"));
}

#[test]
fn nondeterministic_random_gcode_is_fixed_before_shared_export() {
    let machine = serde_json::from_value(serde_json::json!({
        "machine_start_gcode": "G1 X60 Y{random(2,8)}\nG1 X-145 Y{random(-160, -152)}"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &serde_json::Map::new());

    assert_eq!(
        overrides["machine_start_gcode"],
        "G1 X60 Y{2}\nG1 X-145 Y{-160}"
    );
}

#[test]
fn standalone_orca_range_failures_are_normalized_for_shared_smoke_input() {
    let machine = serde_json::from_value(serde_json::json!({
        "retraction_distances_when_cut": ["0", "30"],
        "extruder_printable_height": ["2100"],
        "use_firmware_retraction": "1",
        "nozzle_diameter": ["0.4"]
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "wipe": ["1"],
        "bridge_line_width": "0.6"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(
        overrides["retraction_distances_when_cut"],
        serde_json::json!(["10", "18"])
    );
    assert_eq!(
        overrides["extruder_printable_height"],
        serde_json::json!(["1000"])
    );
    assert_eq!(overrides["use_firmware_retraction"], "0");
    assert_eq!(overrides["bridge_line_width"], "0.4");
}

#[test]
fn relative_e_profile_without_reset_gets_layer_reset() {
    let machine = serde_json::from_value(serde_json::json!({
        "use_relative_e_distances": "1"
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "before_layer_change_gcode": ";before"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(
        overrides
            .get("before_layer_change_gcode")
            .and_then(serde_json::Value::as_str),
        Some(";before\nG92 E0")
    );
}

#[test]
fn commented_reset_does_not_satisfy_relative_e_validation() {
    let machine = serde_json::from_value(serde_json::json!({
        "use_relative_e_distances": "1"
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "before_layer_change_gcode": ";G92 E0.0"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(overrides["before_layer_change_gcode"], ";G92 E0.0\nG92 E0");
}

#[test]
fn absolute_e_profile_removes_active_layer_reset() {
    let machine = serde_json::from_value(serde_json::json!({
        "use_relative_e_distances": "0"
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "before_layer_change_gcode": ";before\nG92 E0\n;after"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(overrides["before_layer_change_gcode"], ";before\n;after");
}

#[test]
fn missing_default_widths_are_materialized_for_nozzle_height_boundary() {
    let machine = serde_json::from_value(serde_json::json!({
        "nozzle_diameter": ["0.2"]
    }))
    .unwrap();
    let mut process = serde_json::from_value(serde_json::json!({
        "layer_height": "0.2"
    }))
    .unwrap();

    normalize_process_defaults(&machine, &mut process);

    assert_eq!(process["skin_infill_line_width"], "0");
    assert_eq!(process["skeleton_infill_line_width"], "0");
}

#[test]
fn line_width_not_above_layer_height_uses_auto_width() {
    let machine = serde_json::from_value(serde_json::json!({
        "nozzle_diameter": ["0.2"]
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "layer_height": "0.2",
        "top_surface_line_width": "100%"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(overrides["top_surface_line_width"], "0");
}

#[test]
fn scalar_standalone_orca_range_failure_is_clamped() {
    let machine = serde_json::from_value(serde_json::json!({
        "retraction_distances_when_cut": "30"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &serde_json::Map::new());

    assert_eq!(overrides["retraction_distances_when_cut"], "18");
}

#[test]
fn smoke_overrides_clear_cli_unsafe_bed_exclusion() {
    let overrides = smoke_overrides();
    assert_eq!(
        overrides.get("bed_exclude_area"),
        Some(&serde_json::json!(["0x0"]))
    );
    assert_eq!(overrides.get("post_process"), Some(&serde_json::json!([])));
}

#[test]
fn sweep_lists_instantiated_printers_only() {
    let names = profiles("Afinia").instantiated_machine_names();

    assert!(names.iter().any(|name| name == "Afinia H+1(HS) 0.4 nozzle"));
    assert!(!names.iter().any(|name| name == "Afinia H+1(HS)"));
    assert!(!names.iter().any(|name| name == "fdm_afinia_common"));
}

#[test]
fn instantiated_printer_names_remain_sorted() {
    let names = profiles("Creality").instantiated_machine_names();
    let mut sorted = names.clone();
    sorted.sort();

    assert_eq!(names, sorted);
}

#[test]
fn selection_falls_back_to_an_explicitly_compatible_filament() {
    let profiles = profiles("Afinia");
    let selection = select_printer(&profiles, "Afinia", "Afinia H+1(HS) 0.4 nozzle").unwrap();

    assert_eq!(selection.process, "0.20mm Standard @Afinia H+1(HS)");
    assert!(selection.filaments[0].contains("PLA"));
}

#[test]
fn smoke_selects_one_filament_for_multi_extruder_printers() {
    let profiles = profiles("Snapmaker");
    let selection =
        select_printer(&profiles, "Snapmaker", "Snapmaker A250 Dual (0.4 nozzle)").unwrap();

    assert_eq!(selection.filaments.len(), 1);
}

#[test]
fn incompatible_named_default_falls_back_to_explicit_compatible_process() {
    let profiles = profiles("Artillery");
    let selection = select_printer(&profiles, "Artillery", "Artillery M1 Pro 0.2 nozzle").unwrap();

    assert!(selection.process.contains("M1 Pro 0.2 nozzle"));
    assert!(profiles.process_is_compatible(&selection.process, &selection.printer));
}

#[test]
fn selection_falls_back_to_a_compatible_process() {
    let profiles = profiles("Anker");
    let selection = select_printer(&profiles, "Anker", "Anker M5 0.2 nozzle").unwrap();

    assert!(selection.process.contains("0.2 nozzle @Anker"));
}

#[test]
fn vendor_without_filament_directory_uses_system_library() {
    let profiles = profiles("Voron");
    let selection = select_printer(&profiles, "Voron", "Voron 0.1 0.4 nozzle").unwrap();

    assert_eq!(selection.filaments, ["Generic PLA @System"]);
    assert!(profiles.filament("Generic PLA @System").is_ok());
}

#[test]
fn missing_local_filament_parent_uses_orca_defaults() {
    let profiles = profiles("Z-Bolt");
    let flattened = profiles.filament("Generic PLA @Z-Bolt 0.4 nozzle").unwrap();

    assert!(!flattened.contains_key("inherits"));
    assert_eq!(
        flattened.get("name").and_then(serde_json::Value::as_str),
        Some("Generic PLA @Z-Bolt 0.4 nozzle")
    );
}
