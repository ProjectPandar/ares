use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn threshold(
    key: &str,
    value: Value,
) -> Result<super::super::support_threshold::SupportThresholdOptions, SliceError> {
    options(json!({ key: value })).support_threshold_options()
}

fn independent_layer_height(value: Value) -> Result<bool, SliceError> {
    Ok(threshold("independent_support_layer_height", value)?.independent_layer_height())
}

fn angle_degrees(value: Value) -> Result<u32, SliceError> {
    Ok(threshold("support_threshold_angle", value)?.angle_degrees())
}

fn overlap(
    value: Value,
) -> Result<super::super::support_threshold::SupportThresholdOverlap, SliceError> {
    Ok(threshold("support_threshold_overlap", value)?.overlap())
}

#[test]
fn support_threshold_options_default_to_orca_values() {
    let threshold = SliceOptions::default().support_threshold_options().unwrap();

    assert!(threshold.independent_layer_height());
    assert_eq!(threshold.angle_degrees(), 30);
    assert_eq!(
        threshold.overlap(),
        super::super::support_threshold::SupportThresholdOverlap::Percent(50.0)
    );
}

#[test]
fn independent_support_layer_height_accepts_booleans() {
    assert!(independent_layer_height(json!(true)).unwrap());
    assert!(!independent_layer_height(json!(false)).unwrap());
}

#[test]
fn independent_support_layer_height_rejects_non_booleans() {
    for value in [
        json!("true"),
        json!("false"),
        json!(1),
        json!(0),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = independent_layer_height(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("independent_support_layer_height"));
    }
}

#[test]
fn support_threshold_angle_accepts_integer_boundaries_and_strings() {
    for (value, expected) in [
        (json!(0), 0),
        (json!("0"), 0),
        (json!(45), 45),
        (json!("45"), 45),
        (json!(90), 90),
        (json!("90"), 90),
    ] {
        assert_eq!(angle_degrees(value).unwrap(), expected);
    }
}

#[test]
fn support_threshold_angle_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!(91),
        json!(45.5),
        json!("45.5"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("invalid"),
        json!(true),
        json!(false),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = angle_degrees(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_threshold_angle"));
    }
}

#[test]
fn support_threshold_overlap_accepts_absolute_values_and_preserves_form() {
    for (value, expected) in [
        (
            json!(0),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(0.0),
        ),
        (
            json!("0"),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(0.0),
        ),
        (
            json!(12.5),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(12.5),
        ),
        (
            json!("12.5"),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(12.5),
        ),
        (
            json!(100.0),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(100.0),
        ),
        (
            json!("100.0"),
            super::super::support_threshold::SupportThresholdOverlap::AbsoluteMm(100.0),
        ),
    ] {
        assert_eq!(overlap(value).unwrap(), expected);
    }
}

#[test]
fn support_threshold_overlap_accepts_percent_values_and_preserves_form() {
    for (value, expected) in [
        (
            json!("0%"),
            super::super::support_threshold::SupportThresholdOverlap::Percent(0.0),
        ),
        (
            json!("50%"),
            super::super::support_threshold::SupportThresholdOverlap::Percent(50.0),
        ),
        (
            json!("100%"),
            super::super::support_threshold::SupportThresholdOverlap::Percent(100.0),
        ),
    ] {
        assert_eq!(overlap(value).unwrap(), expected);
    }
}

#[test]
fn support_threshold_overlap_rejects_invalid_values() {
    for value in [
        json!(-0.001),
        json!(100.001),
        json!("-0.1%"),
        json!("100.001%"),
        json!("1%%"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("bad%"),
        json!(true),
        json!(false),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = overlap(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_threshold_overlap"));
    }
}
