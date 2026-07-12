use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn parsed(
    key: &str,
    value: Value,
) -> Result<super::super::support_z_distance::SupportZDistanceOptions, SliceError> {
    options(json!({ key: value })).support_z_distance_options()
}

#[test]
fn support_z_distance_options_default_to_orca_values() {
    let support = SliceOptions::default().support_z_distance_options().unwrap();

    assert_eq!(support.top_z_distance_mm(), 0.2);
    assert_eq!(support.bottom_z_distance_mm(), 0.2);
    assert_eq!(support.enforce_support_layers(), 0);
    assert!(!support.zero_top_contact());
    assert!(!support.zero_gap_interface_top(1));
    assert!(!support.zero_gap_interface_bottom(1, -1));
}

#[test]
fn support_z_distances_accept_numbers_and_strings() {
    assert_eq!(
        parsed("support_top_z_distance", json!(0.35))
            .unwrap()
            .top_z_distance_mm(),
        0.35
    );
    assert_eq!(
        parsed("support_top_z_distance", json!("0.35"))
            .unwrap()
            .top_z_distance_mm(),
        0.35
    );
    assert_eq!(
        parsed("support_bottom_z_distance", json!(0.45))
            .unwrap()
            .bottom_z_distance_mm(),
        0.45
    );
    assert_eq!(
        parsed("support_bottom_z_distance", json!("0.45"))
            .unwrap()
            .bottom_z_distance_mm(),
        0.45
    );
}

#[test]
fn enforce_support_layers_accepts_strict_integer_values() {
    assert_eq!(
        parsed("enforce_support_layers", json!(7))
            .unwrap()
            .enforce_support_layers(),
        7
    );
    assert_eq!(
        parsed("enforce_support_layers", json!("7"))
            .unwrap()
            .enforce_support_layers(),
        7
    );
}

#[test]
fn support_z_distance_options_accept_boundaries() {
    assert_eq!(
        parsed("support_top_z_distance", json!(0.0))
            .unwrap()
            .top_z_distance_mm(),
        0.0
    );
    assert_eq!(
        parsed("support_bottom_z_distance", json!(0.0))
            .unwrap()
            .bottom_z_distance_mm(),
        0.0
    );
    assert_eq!(
        parsed("enforce_support_layers", json!(0))
            .unwrap()
            .enforce_support_layers(),
        0
    );
    assert_eq!(
        parsed("enforce_support_layers", json!(5000))
            .unwrap()
            .enforce_support_layers(),
        5000
    );
}

#[test]
fn support_z_distance_options_derive_zero_gap_state() {
    let top_zero = parsed("support_top_z_distance", json!(0.0)).unwrap();
    assert!(top_zero.zero_top_contact());
    assert!(top_zero.zero_gap_interface_top(1));
    assert!(!top_zero.zero_gap_interface_top(0));
    assert!(top_zero.zero_gap_interface_bottom(2, -1));
    assert!(!top_zero.zero_gap_interface_bottom(0, -1));
    assert!(top_zero.zero_gap_interface_bottom(1, 2));

    let bottom_zero = parsed("support_bottom_z_distance", json!(0.0)).unwrap();
    assert!(!bottom_zero.zero_gap_interface_top(1));
    assert!(bottom_zero.zero_gap_interface_bottom(1, 2));
    assert!(!bottom_zero.zero_gap_interface_bottom(1, 0));
}

#[test]
fn invalid_support_z_distance_values_reject_with_key() {
    for key in ["support_top_z_distance", "support_bottom_z_distance"] {
        for value in [
            json!(-0.1),
            json!("NaN"),
            json!("inf"),
            json!("fast"),
            json!(true),
            Value::Null,
            json!([]),
            json!({ "value": 0.2 }),
        ] {
            let err = parsed(key, value).unwrap_err();
            assert!(matches!(err, SliceError::InvalidInput(_)));
            assert!(err.to_string().contains(key));
        }
    }
}

#[test]
fn invalid_enforce_support_layers_values_reject_with_key() {
    for value in [
        json!(-1),
        json!(5001),
        json!(1.5),
        json!(5.0),
        json!("1.0"),
        json!("fast"),
        json!(true),
        Value::Null,
        json!([]),
        json!({ "value": 1 }),
    ] {
        let err = parsed("enforce_support_layers", value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("enforce_support_layers"));
    }
}
