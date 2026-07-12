use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn sparse_options_materialize_filament_defaults() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    options.set_num_filaments(3).unwrap();

    assert_eq!(
        options.values()["filament_diameter"],
        json!([1.75, 1.75, 1.75])
    );
    assert_eq!(
        options.values()["filament_colour"],
        json!(["#F2754E", "#F2754E", "#F2754E"])
    );
    assert_eq!(options.values()["wipe"], json!([false, false, false]));
    assert!(!options.values().contains_key("default_filament_profile"));
}

#[test]
fn present_arrays_extend_with_first_entry_and_truncate() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": [2.85],
        "z_hop": [0.2, 0.3, 0.4],
        "wipe": [true]
    }))
    .unwrap();

    options.set_num_filaments(2).unwrap();

    assert_eq!(options.values()["filament_diameter"], json!([2.85, 2.85]));
    assert_eq!(options.values()["z_hop"], json!([0.2, 0.3]));
    assert_eq!(options.values()["wipe"], json!([true, true]));
}

#[test]
fn empty_present_arrays_extend_from_registry_defaults() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": [],
        "filament_colour": [],
        "wipe": []
    }))
    .unwrap();

    options.set_num_filaments(2).unwrap();

    assert_eq!(options.values()["filament_diameter"], json!([1.75, 1.75]));
    assert_eq!(
        options.values()["filament_colour"],
        json!(["#F2754E", "#F2754E"])
    );
    assert_eq!(options.values()["wipe"], json!([false, false]));
}

#[test]
fn default_filament_profile_is_skipped() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "default_filament_profile": ["PLA"]
    }))
    .unwrap();

    options.set_num_filaments(3).unwrap();

    assert_eq!(options.values()["default_filament_profile"], json!(["PLA"]));
}

#[test]
fn zero_filaments_produces_empty_filament_arrays() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": [1.75],
        "filament_colour": ["red"],
        "wipe": [true],
        "default_filament_profile": ["PLA"]
    }))
    .unwrap();

    options.set_num_filaments(0).unwrap();

    assert_eq!(options.values()["filament_diameter"], json!([]));
    assert_eq!(options.values()["filament_colour"], json!([]));
    assert_eq!(options.values()["wipe"], json!([]));
    assert_eq!(options.values()["default_filament_profile"], json!(["PLA"]));
}

#[test]
fn invalid_present_filament_option_returns_invalid_input() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": 1.75,
        "unrelated": "preserved"
    }))
    .unwrap();

    let error = options.set_num_filaments(1).unwrap_err();

    assert!(matches!(error, SliceError::InvalidInput(_)));
    assert_eq!(options.values()["unrelated"], json!("preserved"));
}

#[test]
fn set_num_extruders_still_uses_shared_resize_semantics() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": []
    }))
    .unwrap();

    options.set_num_extruders(2).unwrap();

    assert_eq!(options.values()["nozzle_diameter"], json!([0.4, 0.4]));
}
