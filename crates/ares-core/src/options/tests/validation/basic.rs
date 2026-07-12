use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_basic_fdm_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn invalid_layer_height_is_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["layer_height"].contains("invalid value 0"));
}

#[test]
fn layer_height_modulus_predicate_matches_upstream() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2000005
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(!errors.contains_key("layer_height"));
}

#[test]
fn invalid_initial_layer_height_is_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "initial_layer_print_height": -0.1
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["initial_layer_print_height"].contains("invalid value -0.1"));
}

#[test]
fn invalid_filament_and_nozzle_diameters_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": [1.75, 0.9],
        "nozzle_diameter": [0.4, 0.004]
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["filament_diameter"].contains("0.9"));
    assert!(errors["nozzle_diameter"].contains("0.004"));
}

#[test]
fn string_vector_inputs_match_existing_numeric_parser() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": "1.75,0.9",
        "nozzle_diameter": "0.4;0.004"
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["filament_diameter"].contains("0.9"));
    assert!(errors["nozzle_diameter"].contains("0.004"));
}

#[test]
fn negative_shell_and_wall_counts_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_loops": -1,
        "top_shell_layers": -2,
        "bottom_shell_layers": -3
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["wall_loops"].contains("invalid value -1"));
    assert!(errors["top_shell_layers"].contains("invalid value -2"));
    assert!(errors["bottom_shell_layers"].contains("invalid value -3"));
}

#[test]
fn multiple_invalid_values_accumulate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2,
        "filament_diameter": [0.5],
        "wall_loops": -1
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert_eq!(errors.len(), 3);
    assert!(errors.contains_key("layer_height"));
    assert!(errors.contains_key("filament_diameter"));
    assert!(errors.contains_key("wall_loops"));
}

#[test]
fn invalid_value_types_return_invalid_input() {
    for value in [json!(true), json!([0.2]), json!({"height": 0.2})] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": value
        }))
        .unwrap();

        let error = options.validate_basic_fdm_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn count_resize_apis_remain_intact() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "nozzle_diameter": [],
        "filament_diameter": []
    }))
    .unwrap();

    options.set_num_extruders(2).unwrap();
    options.set_num_filaments(2).unwrap();

    assert_eq!(options.values()["nozzle_diameter"], json!([0.4, 0.4]));
    assert_eq!(options.values()["filament_diameter"], json!([1.75, 1.75]));
}
