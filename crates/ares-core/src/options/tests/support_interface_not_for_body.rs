use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn support_interface_not_for_body(value: Value) -> Result<bool, SliceError> {
    Ok(options(json!({ "support_interface_not_for_body": value }))
        .support_interface_not_for_body_options()?
        .not_for_body())
}

#[test]
fn support_interface_not_for_body_defaults_to_true() {
    assert!(SliceOptions::default()
        .support_interface_not_for_body_options()
        .unwrap()
        .not_for_body());
}

#[test]
fn support_interface_not_for_body_accepts_booleans() {
    assert!(support_interface_not_for_body(json!(true)).unwrap());
    assert!(!support_interface_not_for_body(json!(false)).unwrap());
}

#[test]
fn support_interface_not_for_body_rejects_non_booleans() {
    for value in [
        json!("true"),
        json!("false"),
        json!(1),
        json!(0),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = support_interface_not_for_body(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_not_for_body"));
    }
}
