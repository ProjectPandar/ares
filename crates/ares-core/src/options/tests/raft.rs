use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn raft_layers(value: Value) -> Result<u32, SliceError> {
    Ok(options(json!({ "raft_layers": value }))
        .raft_options()?
        .layers())
}

#[test]
fn raft_layers_default_to_orca_value() {
    let raft = SliceOptions::default().raft_options().unwrap();

    assert_eq!(raft.layers(), 0);
    assert!(!raft.has_raft());
}

#[test]
fn raft_layers_accept_integer_boundaries_and_strings() {
    assert_eq!(raft_layers(json!(0)).unwrap(), 0);
    assert_eq!(raft_layers(json!("0")).unwrap(), 0);
    assert_eq!(raft_layers(json!(7)).unwrap(), 7);
    assert_eq!(raft_layers(json!("7")).unwrap(), 7);
    assert_eq!(raft_layers(json!(100)).unwrap(), 100);
    assert_eq!(raft_layers(json!("100")).unwrap(), 100);
}

#[test]
fn positive_raft_layers_report_has_raft() {
    let raft = options(json!({ "raft_layers": 1 })).raft_options().unwrap();

    assert_eq!(raft.layers(), 1);
    assert!(raft.has_raft());
}

#[test]
fn raft_layers_reject_invalid_values() {
    for value in [
        json!(-1),
        json!(1.5),
        json!("1.5"),
        json!("bad"),
        json!(101),
        json!("101"),
        json!(true),
        Value::Null,
    ] {
        let err = raft_layers(value).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_layers"));
    }
}

fn raft_expansion(value: Value) -> Result<f64, SliceError> {
    options(json!({ "raft_expansion": value })).raft_expansion_mm()
}

fn raft_first_layer_expansion(value: Value) -> Result<f64, SliceError> {
    options(json!({ "raft_first_layer_expansion": value })).raft_first_layer_expansion_mm()
}

fn raft_first_layer_density(value: Value) -> Result<f64, SliceError> {
    options(json!({ "raft_first_layer_density": value })).raft_first_layer_density_percent()
}

#[test]
fn raft_expansion_defaults_to_orca_value() {
    assert_eq!(SliceOptions::default().raft_expansion_mm().unwrap(), 1.5);
}

#[test]
fn raft_expansion_accepts_non_negative_numbers_and_strings() {
    assert_eq!(raft_expansion(json!(0)).unwrap(), 0.0);
    assert_eq!(raft_expansion(json!("0")).unwrap(), 0.0);
    assert_eq!(raft_expansion(json!(0.75)).unwrap(), 0.75);
    assert_eq!(raft_expansion(json!("0.75")).unwrap(), 0.75);
    assert_eq!(raft_expansion(json!(1.5)).unwrap(), 1.5);
}

#[test]
fn raft_expansion_rejects_invalid_values() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 0.5 }),
        json!(true),
        Value::Null,
    ] {
        let err = raft_expansion(value).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_expansion"));
    }
}

#[test]
fn raft_first_layer_expansion_defaults_to_orca_value() {
    assert_eq!(
        SliceOptions::default()
            .raft_first_layer_expansion_mm()
            .unwrap(),
        2.0
    );
}

#[test]
fn raft_first_layer_expansion_accepts_non_negative_numbers_and_strings() {
    assert_eq!(raft_first_layer_expansion(json!(0)).unwrap(), 0.0);
    assert_eq!(raft_first_layer_expansion(json!("0")).unwrap(), 0.0);
    assert_eq!(raft_first_layer_expansion(json!(0.75)).unwrap(), 0.75);
    assert_eq!(raft_first_layer_expansion(json!("0.75")).unwrap(), 0.75);
    assert_eq!(raft_first_layer_expansion(json!(2)).unwrap(), 2.0);
}

#[test]
fn raft_first_layer_expansion_rejects_invalid_values() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 0.5 }),
        json!(true),
        Value::Null,
    ] {
        let err = raft_first_layer_expansion(value).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_first_layer_expansion"));
    }
}

#[test]
fn raft_first_layer_density_defaults_to_orca_value() {
    assert_eq!(
        SliceOptions::default()
            .raft_first_layer_density_percent()
            .unwrap(),
        90.0
    );
}

#[test]
fn raft_first_layer_density_accepts_numeric_boundaries_and_strings() {
    assert_eq!(raft_first_layer_density(json!(10.0)).unwrap(), 10.0);
    assert_eq!(raft_first_layer_density(json!("10.0")).unwrap(), 10.0);
    assert_eq!(raft_first_layer_density(json!(50.5)).unwrap(), 50.5);
    assert_eq!(raft_first_layer_density(json!("50.5")).unwrap(), 50.5);
    assert_eq!(raft_first_layer_density(json!(100.0)).unwrap(), 100.0);
    assert_eq!(raft_first_layer_density(json!("100.0")).unwrap(), 100.0);
}

#[test]
fn raft_first_layer_density_rejects_invalid_values() {
    for value in [
        json!(9.99),
        json!("9.99"),
        json!(100.01),
        json!("100.01"),
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("50%"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 50.0 }),
        json!(true),
        Value::Null,
    ] {
        let err = raft_first_layer_density(value).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_first_layer_density"));
    }
}
