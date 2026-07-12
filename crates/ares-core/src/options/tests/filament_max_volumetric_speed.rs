use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

#[test]
fn omitted_filament_max_volumetric_speed_uses_orca_default() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert_eq!(speeds.filament_max_volumetric_speed_mm3_s(), 2.0);
}

#[test]
fn parsed_filament_max_volumetric_speed_reaches_speed_options() {
    for (value, expected) in [
        (json!(8.5), 8.5),
        (json!("9.25"), 9.25),
        (json!([10.5, 2.0]), 10.5),
        (json!("11.75;2.0"), 11.75),
        (json!("12.5,2.0"), 12.5),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_max_volumetric_speed": value })).unwrap();

        let speeds = options.speed_options().unwrap();

        assert_eq!(speeds.filament_max_volumetric_speed_mm3_s(), expected);
    }
}

#[test]
fn zero_filament_max_volumetric_speed_is_allowed() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "filament_max_volumetric_speed": 0.0 })).unwrap();

    let speeds = options.speed_options().unwrap();

    assert_eq!(speeds.filament_max_volumetric_speed_mm3_s(), 0.0);
}

#[test]
fn speed_options_carry_first_filament_diameter() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "filament_diameter": [2.85, 1.75] })).unwrap();

    let speeds = options.speed_options().unwrap();

    assert_eq!(speeds.filament_diameter_mm(), 2.85);
}

#[test]
fn adaptive_volumetric_speed_options_reach_speed_options() {
    for (enabled, coefficients) in [
        (json!(true), json!("0 0 0 0 0 1")),
        (json!([true, false]), json!(["0  0 0 0 0 1", "0 0 0 0 0 2"])),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_adaptive_volumetric_speed": enabled,
            "volumetric_speed_coefficients": coefficients
        }))
        .unwrap();

        let speeds = options.speed_options().unwrap();

        assert!(speeds.filament_adaptive_volumetric_speed());
        assert_eq!(
            speeds.volumetric_speed_coefficients(),
            Some([0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
        );
    }
}

#[test]
fn adaptive_volumetric_speed_defaults_to_disabled_without_coefficients() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert!(!speeds.filament_adaptive_volumetric_speed());
    assert_eq!(speeds.volumetric_speed_coefficients(), None);
}

#[test]
fn extrusion_rate_smoothing_options_reach_speed_options() {
    for (
        slope,
        segment_length,
        external_only,
        expected_slope,
        expected_segment_length,
        expected_external_only,
    ) in [
        (json!(12.5), json!(2.5), json!(true), 12.5, 2.5, true),
        (json!("7.25"), json!("0.5"), json!(true), 7.25, 0.5, true),
        (json!(0.0), json!(5.0), json!(true), 0.0, 5.0, true),
        (json!("0"), json!(3.0), json!(false), 0.0, 3.0, false),
        (json!("0.0"), json!("5.0"), json!(true), 0.0, 5.0, true),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "max_volumetric_extrusion_rate_slope": slope,
            "max_volumetric_extrusion_rate_slope_segment_length": segment_length,
            "extrusion_rate_smoothing_external_perimeter_only": external_only
        }))
        .unwrap();

        let speeds = options.speed_options().unwrap();

        assert_eq!(
            speeds.max_volumetric_extrusion_rate_slope_mm3_s2(),
            expected_slope
        );
        assert_eq!(
            speeds.max_volumetric_extrusion_rate_slope_segment_length_mm(),
            expected_segment_length
        );
        assert_eq!(
            speeds.extrusion_rate_smoothing_external_perimeter_only(),
            expected_external_only
        );
    }
}

#[test]
fn extrusion_rate_smoothing_defaults_match_orca() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert_eq!(speeds.max_volumetric_extrusion_rate_slope_mm3_s2(), 0.0);
    assert_eq!(
        speeds.max_volumetric_extrusion_rate_slope_segment_length_mm(),
        3.0
    );
    assert!(!speeds.extrusion_rate_smoothing_external_perimeter_only());
}

#[test]
fn adaptive_volumetric_speed_ignores_unusable_coefficients() {
    for coefficients in [
        json!(""),
        json!("0 0 0"),
        json!("0 0 0 0 0 0"),
        json!("0 0 0 0 0 NaN"),
        json!("0\t0 0 0 0 1"),
        json!("0\n0 0 0 0 1"),
        json!("bad 0 0 0 0 1"),
        json!([""]),
        json!(["0\t0 0 0 0 1"]),
        json!(["0\n0 0 0 0 1"]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_adaptive_volumetric_speed": true,
            "volumetric_speed_coefficients": coefficients
        }))
        .unwrap();

        let speeds = options.speed_options().unwrap();

        assert!(speeds.filament_adaptive_volumetric_speed());
        assert_eq!(speeds.volumetric_speed_coefficients(), None);
    }
}

#[test]
fn rejects_invalid_adaptive_volumetric_speed_flag() {
    for value in [json!("true"), json!(1), json!([]), json!([1]), Value::Null] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_adaptive_volumetric_speed": value
        }))
        .unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("filament_adaptive_volumetric_speed")
        );
    }
}

#[test]
fn rejects_invalid_filament_max_volumetric_speed_values() {
    for value in [
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
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_max_volumetric_speed": value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_max_volumetric_speed"));
    }
}

#[test]
fn rejects_invalid_extrusion_rate_smoothing_values() {
    for (key, value) in [
        ("max_volumetric_extrusion_rate_slope", json!(-0.1)),
        ("max_volumetric_extrusion_rate_slope", json!("NaN")),
        ("max_volumetric_extrusion_rate_slope", json!("inf")),
        ("max_volumetric_extrusion_rate_slope", json!("-inf")),
        ("max_volumetric_extrusion_rate_slope", json!("")),
        ("max_volumetric_extrusion_rate_slope", json!("not-a-number")),
        ("max_volumetric_extrusion_rate_slope", json!([])),
        (
            "max_volumetric_extrusion_rate_slope",
            json!({ "value": 1.0 }),
        ),
        ("max_volumetric_extrusion_rate_slope", json!(true)),
        ("max_volumetric_extrusion_rate_slope", Value::Null),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!(0.49),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!(5.01),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!("NaN"),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!("inf"),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!("-inf"),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!(""),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!("not-a-number"),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!([]),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!({ "value": 1.0 }),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!(true),
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            Value::Null,
        ),
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            json!("true"),
        ),
        ("extrusion_rate_smoothing_external_perimeter_only", json!(1)),
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            json!([]),
        ),
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            json!({ "value": true }),
        ),
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            Value::Null,
        ),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
