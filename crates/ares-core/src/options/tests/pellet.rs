use super::super::*;
use crate::{PrintPathRole, SliceError};
use serde_json::{Value, json};

#[test]
fn pellet_modded_printer_absent_or_false_preserves_filament_diameter() {
    let absent: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": [1.75, 2.85],
        "pellet_flow_coefficient": [1.0]
    }))
    .unwrap();
    let disabled: SliceOptions = serde_json::from_value(json!({
        "pellet_modded_printer": false,
        "filament_diameter": [1.75, 2.85],
        "pellet_flow_coefficient": [1.0]
    }))
    .unwrap();

    assert_eq!(absent.filament_diameters().unwrap(), vec![1.75, 2.85]);
    assert_eq!(disabled.filament_diameters().unwrap(), vec![1.75, 2.85]);
}

#[test]
fn pellet_modded_printer_converts_default_coefficient_to_effective_diameter() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "pellet_modded_printer": true })).unwrap();

    let diameters = options.filament_diameters().unwrap();

    assert_eq!(diameters.len(), 1);
    assert_close(diameters[0], 1.7501087939894036);
}

#[test]
fn pellet_flow_coefficient_accepts_orca_numeric_vector_forms() {
    for (value, expected) in [
        (json!(std::f64::consts::FRAC_1_PI), vec![2.0]),
        (json!("0.3183098861837907"), vec![2.0]),
        (
            json!("0.3183098861837907;1.2732395447351628"),
            vec![2.0, 1.0],
        ),
        (
            json!("0.3183098861837907,1.2732395447351628"),
            vec![2.0, 1.0],
        ),
        (
            json!(["0.3183098861837907", 1.2732395447351628]),
            vec![2.0, 1.0],
        ),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "pellet_modded_printer": true,
            "pellet_flow_coefficient": value
        }))
        .unwrap();

        let diameters = options.filament_diameters().unwrap();

        assert_close_vec(&diameters, &expected);
    }
}

#[test]
fn pellet_mode_feeds_existing_hardware_extrusion_and_speed_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "pellet_modded_printer": true,
        "pellet_flow_coefficient": std::f64::consts::FRAC_1_PI,
        "filament_diameter": [9.99],
        "nozzle_diameter": [0.4],
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "sparse_infill_line_width": 0.4,
        "filament_max_volumetric_speed": 0.0
    }))
    .unwrap();

    let hardware = options.hardware_options().unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let speeds = options.speed_options().unwrap();

    assert_close_vec(hardware.filament_diameters(), &[2.0]);
    assert_close(extrusion.filament_diameter(), 2.0);
    assert_close(speeds.filament_diameter_mm(), 2.0);
    assert_close(
        extrusion
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap(),
        0.02273239544735163,
    );
}

#[test]
fn invalid_pellet_modded_printer_is_rejected() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "pellet_modded_printer": "true" })).unwrap();

    let err = options.filament_diameters().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("pellet_modded_printer must be a boolean")
    );
}

#[test]
fn invalid_pellet_flow_coefficient_values_are_rejected() {
    for value in [
        json!(0.0),
        json!(-0.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("1.0;bad"),
        json!(""),
        json!([]),
        json!([1.0, "bad"]),
        json!({"value": 1.0}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "pellet_modded_printer": true,
            "pellet_flow_coefficient": value
        }))
        .unwrap();

        let err = options.filament_diameters().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("pellet_flow_coefficient"));
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 0.000000001,
        "{actual} != {expected}"
    );
}

fn assert_close_vec(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_close(*actual, *expected);
    }
}
