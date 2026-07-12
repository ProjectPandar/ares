use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn sparse_options_materialize_extruder_defaults_and_extend_variants() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    options.set_num_extruders(3).unwrap();

    assert_eq!(
        options.values()["extruder_variant_list"],
        json!([
            "Direct Drive Standard",
            "Direct Drive Standard",
            "Direct Drive Standard"
        ])
    );
    assert_eq!(options.values()["printer_extruder_id"], json!([1, 2, 3]));
    assert_eq!(options.values()["nozzle_diameter"], json!([0.4, 0.4, 0.4]));
    assert_eq!(options.values()["wipe"], json!([false, false, false]));
    assert_eq!(
        options.values()["extruder_type"],
        json!(["Direct Drive", "Direct Drive", "Direct Drive"])
    );
    assert!(!options.values().contains_key("default_filament_profile"));
}

#[test]
fn present_arrays_extend_with_first_entry_and_truncate() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": [0.6],
        "z_hop": [0.2, 0.3, 0.4],
        "wipe": [true]
    }))
    .unwrap();

    options.set_num_extruders(2).unwrap();

    assert_eq!(options.values()["nozzle_diameter"], json!([0.6, 0.6]));
    assert_eq!(options.values()["z_hop"], json!([0.2, 0.3]));
    assert_eq!(options.values()["wipe"], json!([true, true]));
}

#[test]
fn empty_present_arrays_extend_from_registry_defaults() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": [],
        "wipe": [],
        "extruder_type": []
    }))
    .unwrap();

    options.set_num_extruders(2).unwrap();

    assert_eq!(options.values()["nozzle_diameter"], json!([0.4, 0.4]));
    assert_eq!(options.values()["wipe"], json!([false, false]));
    assert_eq!(
        options.values()["extruder_type"],
        json!(["Direct Drive", "Direct Drive"])
    );
}

#[test]
fn variant_sized_keys_use_parameter_size_after_variant_extension() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A,B", "C"],
        "nozzle_type": []
    }))
    .unwrap();

    options.set_num_extruders(2).unwrap();

    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!(["A", "B", "C"])
    );
    assert_eq!(
        options.values()["nozzle_type"],
        json!(["undefine", "undefine", "undefine"])
    );
}

#[test]
fn default_filament_profile_is_skipped() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "default_filament_profile": ["PLA"]
    }))
    .unwrap();

    options.set_num_extruders(3).unwrap();

    assert_eq!(options.values()["default_filament_profile"], json!(["PLA"]));
}

#[test]
fn zero_extruders_produces_empty_extruder_arrays() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": [0.4],
        "wipe": [true],
        "default_filament_profile": ["PLA"]
    }))
    .unwrap();

    options.set_num_extruders(0).unwrap();

    assert_eq!(options.values()["extruder_variant_list"], json!([]));
    assert_eq!(options.values()["printer_extruder_id"], json!([]));
    assert_eq!(options.values()["printer_extruder_variant"], json!([]));
    assert_eq!(options.values()["nozzle_diameter"], json!([]));
    assert_eq!(options.values()["wipe"], json!([]));
    assert_eq!(options.values()["default_filament_profile"], json!(["PLA"]));
}

#[test]
fn invalid_present_extruder_option_returns_invalid_input() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": 0.4,
        "unrelated": "preserved"
    }))
    .unwrap();

    let error = options.set_num_extruders(1).unwrap_err();

    assert!(matches!(error, SliceError::InvalidInput(_)));
    assert_eq!(options.values()["unrelated"], json!("preserved"));
}
