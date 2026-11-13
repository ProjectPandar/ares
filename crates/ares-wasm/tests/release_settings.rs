use std::{fs, path::Path};

use ares_core::{FilamentOptions, PrinterOptions, ProcessOptions};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn defaults() -> Value {
    serde_json::from_str(include_str!("release_settings/defaults.json")).unwrap()
}

fn assert_fields(actual: &Value, expected: &Value) {
    for (key, value) in expected.as_object().unwrap() {
        assert_eq!(actual.get(key), Some(value), "{key}");
    }
}

#[test]
fn release_defaults_are_available_in_each_public_option_map() {
    let expected = defaults();
    assert_fields(
        &serde_json::to_value(ProcessOptions::default()).unwrap(),
        &expected["process"],
    );
    assert_fields(
        &serde_json::to_value(FilamentOptions::default()).unwrap(),
        &expected["filament"],
    );
    assert_fields(
        &serde_json::to_value(PrinterOptions::default()).unwrap(),
        &expected["printer"],
    );
}

fn retain<T: DeserializeOwned + Serialize>(input: Value) {
    let options: T = serde_json::from_value(input.clone()).unwrap();
    assert_fields(&serde_json::to_value(options).unwrap(), &input);
}

#[test]
fn release_fields_retain_non_default_values_through_public_options() {
    let values: Value = serde_json::from_str(include_str!("release_settings/values.json")).unwrap();
    retain::<ProcessOptions>(values["process"].clone());
    retain::<FilamentOptions>(values["filament"].clone());
    retain::<PrinterOptions>(values["printer"].clone());
}

#[test]
fn printer_frame_and_process_enums_retain_every_upstream_token() {
    for token in ["none", "x", "y", "z"] {
        retain::<PrinterOptions>(json!({"belt_slice_rotation": token}));
    }
    for token in [
        "pos_x", "pos_y", "pos_z", "neg_x", "neg_y", "neg_z", "rev_x", "rev_y", "rev_z",
    ] {
        retain::<PrinterOptions>(
            json!({"gcode_remap_x": token, "gcode_remap_y": token, "gcode_remap_z": token}),
        );
    }
    for token in ["undefine", "left", "right", "both"] {
        retain::<PrinterOptions>(json!({"fan_direction": token}));
    }
    for token in ["standard", "deuteranopia", "tritanopia", "high_contrast"] {
        retain::<PrinterOptions>(json!({"imex_viz_theme": token}));
    }
    for token in ["default", "outward", "inward"] {
        retain::<ProcessOptions>(
            json!({"top_surface_fill_order": token, "bottom_surface_fill_order": token}),
        );
    }
    for token in ["each_surface", "each_model", "each_assembly"] {
        retain::<ProcessOptions>(json!({"center_of_surface_pattern": token}));
    }
    for token in [
        "disabled",
        "distance_warp",
        "smooth_blend",
        "stepped_shells",
        "lobes",
        "normal_z",
        "normal_y",
        "normal_x",
    ] {
        retain::<ProcessOptions>(json!({"adaptive_infill_mode": token}));
    }
    for token in ["linear", "quadratic", "exponential"] {
        retain::<ProcessOptions>(json!({"adaptive_infill_gradient": token}));
    }
    for token in ["default", "cyclic"] {
        retain::<ProcessOptions>(json!({"toolchange_ordering": token}));
    }
    assert!(serde_json::from_value::<PrinterOptions>(json!({"gcode_remap_x": "pos_u"})).is_err());
    assert!(
        serde_json::from_value::<ProcessOptions>(json!({"adaptive_infill_mode": "radial"}))
            .is_err()
    );
}

#[test]
fn loaded_scene_exposes_release_rows_without_copying_project_owned_fields() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let scene = ares_wasm::scene_from_bytes(input).unwrap();
    let actual: Value = serde_json::from_str(&scene.metadata()).unwrap();
    let settings = &actual["settings"];
    let expected = defaults();
    for tab in ["process", "filament", "printer"] {
        for key in expected[tab].as_object().unwrap().keys() {
            assert!(settings[tab].get(key).is_some(), "{tab}/{key}");
        }
    }
    for key in [
        "nozzle_diameter",
        "min_layer_height",
        "max_layer_height",
        "extruder_offset",
        "retract_length_toolchange",
        "retract_restart_extra_toolchange",
    ] {
        assert!(settings["project"].get(key).is_some(), "{key}");
        assert!(settings["printer"].get(key).is_none(), "{key}");
    }
    let destination =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/web-e2e/release-settings.json");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(destination, serde_json::to_vec_pretty(settings).unwrap()).unwrap();
}
