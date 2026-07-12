use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn non_zero_extruder_is_erased_and_populates_missing_role_filaments() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("extruder"));
    assert_eq!(options.values()["sparse_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_filament"], json!(3));
    assert_eq!(options.values()["solid_infill_filament"], json!(3));
}

#[test]
fn existing_role_filaments_are_preserved() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
        "sparse_infill_filament": 4,
        "wall_filament": 5,
        "solid_infill_filament": 6,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("extruder"));
    assert_eq!(options.values()["sparse_infill_filament"], json!(4));
    assert_eq!(options.values()["wall_filament"], json!(5));
    assert_eq!(options.values()["solid_infill_filament"], json!(6));
}

#[test]
fn zero_extruder_is_erased_without_role_propagation() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 0,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("extruder"));
    assert!(!options.values().contains_key("sparse_infill_filament"));
    assert!(!options.values().contains_key("wall_filament"));
    assert!(!options.values().contains_key("solid_infill_filament"));
}

#[test]
fn solid_infill_filament_falls_back_to_sparse_infill_filament() {
    for (sparse_value, expected) in [(json!(7), json!(7)), (json!("8"), json!(8))] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "sparse_infill_filament": sparse_value,
        }))
        .unwrap();

        options.normalize_fdm(2).unwrap();

        assert_eq!(options.values()["solid_infill_filament"], expected);
    }
}

#[test]
fn support_filaments_are_not_populated_from_extruder() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("support_filament"));
    assert!(!options.values().contains_key("support_interface_filament"));
}

#[test]
fn invalid_integer_values_return_invalid_input() {
    for (key, value) in [
        ("extruder", json!(-1)),
        ("extruder", json!(1.5)),
        ("extruder", json!("bad")),
        ("extruder", json!(true)),
        ("extruder", json!([1])),
        ("sparse_infill_filament", json!(-1)),
        ("sparse_infill_filament", json!(1.5)),
        ("sparse_infill_filament", json!("bad")),
        ("sparse_infill_filament", json!(true)),
        ("sparse_infill_filament", json!({"value": 1})),
    ] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            key: value,
        }))
        .unwrap();

        let err = options.normalize_fdm(2).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn deserialization_does_not_normalize_fdm_automatically() {
    let options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
        "resolution": 0,
    }))
    .unwrap();

    assert_eq!(options.values()["extruder"], json!(3));
    assert_eq!(options.values()["resolution"], json!(0));
    assert!(!options.values().contains_key("sparse_infill_filament"));
    assert!(!options.values().contains_key("wall_filament"));
    assert!(!options.values().contains_key("solid_infill_filament"));
}

#[test]
fn absent_resolution_remains_absent() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("resolution"));
}

#[test]
fn resolution_below_minimum_clamps_to_orca_lower_bound() {
    for value in [json!(0), json!(0.0005), json!("0"), json!("0.0005")] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "resolution": value,
        }))
        .unwrap();

        options.normalize_fdm(2).unwrap();

        assert_eq!(options.values()["resolution"], json!(0.001));
    }
}

#[test]
fn resolution_at_or_above_minimum_is_preserved_as_number() {
    for (value, expected) in [
        (json!(0.001), json!(0.001)),
        (json!("0.001"), json!(0.001)),
        (json!(0.25), json!(0.25)),
        (json!("0.25"), json!(0.25)),
    ] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "resolution": value,
        }))
        .unwrap();

        options.normalize_fdm(2).unwrap();

        assert_eq!(options.values()["resolution"], expected);
    }
}

#[test]
fn invalid_resolution_values_return_invalid_input() {
    for value in [
        json!(-0.001),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!(true),
        json!(null),
        json!([0.001]),
    ] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "resolution": value,
        }))
        .unwrap();

        let err = options.normalize_fdm(2).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn resolution_clamp_keeps_extruder_and_spiral_normalization() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
        "resolution": 0,
        "spiral_mode": true,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["resolution"], json!(0.001));
    assert!(!options.values().contains_key("extruder"));
    assert_eq!(options.values()["sparse_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_filament"], json!(3));
    assert_eq!(options.values()["solid_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_loops"], json!(1));
}

#[test]
fn absent_spiral_mode_leaves_spiral_options_unchanged() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "retract_when_changing_layer": [true, false],
        "filament_retract_when_changing_layer": [true, null],
        "wall_loops": 3,
        "alternate_extra_wall": true,
        "top_shell_layers": 4,
        "sparse_infill_density": 15,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(
        options.values()["retract_when_changing_layer"],
        json!([true, false])
    );
    assert_eq!(
        options.values()["filament_retract_when_changing_layer"],
        json!([true, null])
    );
    assert_eq!(options.values()["wall_loops"], json!(3));
    assert_eq!(options.values()["alternate_extra_wall"], json!(true));
    assert_eq!(options.values()["top_shell_layers"], json!(4));
    assert_eq!(options.values()["sparse_infill_density"], json!(15));
}

#[test]
fn false_spiral_mode_leaves_spiral_options_unchanged() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": false,
        "retract_when_changing_layer": [true],
        "filament_retract_when_changing_layer": [null],
        "wall_loops": 3,
        "alternate_extra_wall": true,
        "top_shell_layers": 4,
        "sparse_infill_density": 15,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(
        options.values()["retract_when_changing_layer"],
        json!([true])
    );
    assert_eq!(
        options.values()["filament_retract_when_changing_layer"],
        json!([null])
    );
    assert_eq!(options.values()["wall_loops"], json!(3));
    assert_eq!(options.values()["alternate_extra_wall"], json!(true));
    assert_eq!(options.values()["top_shell_layers"], json!(4));
    assert_eq!(options.values()["sparse_infill_density"], json!(15));
}

#[test]
fn true_spiral_mode_forces_scalar_options() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "wall_loops": 3,
        "alternate_extra_wall": true,
        "top_shell_layers": 4,
        "sparse_infill_density": 15,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["wall_loops"], json!(1));
    assert_eq!(options.values()["alternate_extra_wall"], json!(false));
    assert_eq!(options.values()["top_shell_layers"], json!(0));
    assert_eq!(options.values()["sparse_infill_density"], json!(0));
}

#[test]
fn true_spiral_mode_disables_existing_retraction_arrays_and_preserves_lengths() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "retract_when_changing_layer": [true, false, true],
        "filament_retract_when_changing_layer": [true, null, false],
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(
        options.values()["retract_when_changing_layer"],
        json!([false, false, false])
    );
    assert_eq!(
        options.values()["filament_retract_when_changing_layer"],
        json!([false, false, false])
    );
}

#[test]
fn true_spiral_mode_inserts_missing_retraction_arrays() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(
        options.values()["retract_when_changing_layer"],
        json!([false])
    );
    assert_eq!(
        options.values()["filament_retract_when_changing_layer"],
        json!([false])
    );
}

#[test]
fn invalid_spiral_mode_returns_invalid_input() {
    for value in [json!(1), json!("true"), json!(null), json!([true])] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "spiral_mode": value,
        }))
        .unwrap();

        let err = options.normalize_fdm(2).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn invalid_spiral_retraction_arrays_return_invalid_input() {
    for (key, value) in [
        ("retract_when_changing_layer", json!(true)),
        ("retract_when_changing_layer", json!([true, 1])),
        ("filament_retract_when_changing_layer", json!(false)),
        (
            "filament_retract_when_changing_layer",
            json!([null, "false"]),
        ),
    ] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "spiral_mode": true,
            key: value,
        }))
        .unwrap();

        let err = options.normalize_fdm(2).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn spiral_mode_keeps_extruder_role_propagation() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
        "spiral_mode": true,
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert!(!options.values().contains_key("extruder"));
    assert_eq!(options.values()["sparse_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_filament"], json!(3));
    assert_eq!(options.values()["solid_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_loops"], json!(1));
}
