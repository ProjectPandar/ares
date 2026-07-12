use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

const SPIRAL_VASE_MESSAGE_PREFIX: &str = "Invalid value when spiral vase mode is enabled";

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn support_enable(value: Value) -> Result<bool, SliceError> {
    Ok(options(json!({ "enable_support": value }))
        .support_enable_options()?
        .enabled())
}

#[test]
fn support_enable_defaults_to_false() {
    assert!(!SliceOptions::default()
        .support_enable_options()
        .unwrap()
        .enabled());
}

#[test]
fn support_enable_accepts_booleans() {
    assert!(support_enable(json!(true)).unwrap());
    assert!(!support_enable(json!(false)).unwrap());
}

#[test]
fn support_enable_rejects_non_booleans() {
    for value in [
        json!("true"),
        json!("false"),
        json!(1),
        json!(0),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = support_enable(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("enable_support"));
    }
}

#[test]
fn spiral_vase_validation_still_reports_enable_support() {
    let options = options(json!({
        "spiral_mode": true,
        "wall_loops": 1,
        "sparse_infill_density": 0,
        "top_shell_layers": 0,
        "enable_support": true
    }));

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert_eq!(
        errors["enable_support"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 1")
    );
    assert_eq!(errors.len(), 1);
}
