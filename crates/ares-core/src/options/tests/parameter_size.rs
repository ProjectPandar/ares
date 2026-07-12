use serde_json::json;

use crate::{SliceError, SliceOptions};

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn variant_lengths_default_to_one_when_sources_absent() {
    let options = SliceOptions::default();

    assert_eq!(options.parameter_size("deretraction_speed", 7), Ok(1));
    assert_eq!(options.parameter_size("machine_max_speed_z", 7), Ok(2));
    assert_eq!(options.parameter_size("filament_flow_ratio", 7), Ok(1));
    assert_eq!(options.parameter_size("print_extruder_id", 7), Ok(1));
}

#[test]
fn printer_variant_one_keys_use_machine_variant_length() {
    let options = options(json!({
        "printer_extruder_variant": ["standard", "hardened", "high-flow"]
    }));

    assert_eq!(options.parameter_size("deretraction_speed", 9), Ok(3));
}

#[test]
fn printer_variant_two_keys_use_double_machine_variant_length() {
    let options = options(json!({
        "printer_extruder_variant": ["standard", "hardened", "high-flow"]
    }));

    assert_eq!(options.parameter_size("machine_max_speed_z", 9), Ok(6));
}

#[test]
fn filament_variant_keys_use_filament_variant_length() {
    let options = options(json!({
        "filament_extruder_variant": ["pla", "petg", "abs", "asa"]
    }));

    assert_eq!(options.parameter_size("filament_flow_ratio", 9), Ok(4));
}

#[test]
fn print_variant_keys_use_process_variant_length() {
    let options = options(json!({
        "print_extruder_variant": ["draft", "quality", "strong"]
    }));

    assert_eq!(options.parameter_size("print_extruder_id", 9), Ok(3));
}

#[test]
fn fallback_keys_use_extruder_count() {
    let options = options(json!({
        "filament_extruder_variant": ["pla", "petg"],
        "print_extruder_variant": ["draft", "quality", "strong"],
        "printer_extruder_variant": ["standard"]
    }));

    assert_eq!(options.parameter_size("sparse_infill_density", 5), Ok(5));
}

#[test]
fn variant_source_lengths_only_affect_owning_key_sets() {
    let options = options(json!({
        "filament_extruder_variant": ["pla", "petg", "abs", "asa"],
        "print_extruder_variant": ["draft", "quality", "strong"],
        "printer_extruder_variant": ["standard", "hardened"]
    }));

    assert_eq!(options.parameter_size("filament_flow_ratio", 9), Ok(4));
    assert_eq!(options.parameter_size("print_extruder_id", 9), Ok(3));
    assert_eq!(options.parameter_size("deretraction_speed", 9), Ok(2));
    assert_eq!(options.parameter_size("machine_max_speed_z", 9), Ok(4));
}

#[test]
fn invalid_variant_source_values_return_invalid_input() {
    for (key, value) in [
        ("filament_extruder_variant", json!("pla")),
        ("print_extruder_variant", json!(["draft", 2])),
        ("printer_extruder_variant", json!(["standard", null])),
    ] {
        let options = options(json!({ key: value }));
        let error = options
            .parameter_size("sparse_infill_density", 2)
            .unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}
