use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn support_object_skip_flush(value: Value) -> Result<bool, SliceError> {
    Ok(options(json!({ "support_object_skip_flush": value }))
        .support_object_skip_flush_options()?
        .skip_flush())
}

#[test]
fn support_object_skip_flush_defaults_to_false() {
    assert!(!SliceOptions::default()
        .support_object_skip_flush_options()
        .unwrap()
        .skip_flush());
}

#[test]
fn support_object_skip_flush_accepts_booleans() {
    assert!(support_object_skip_flush(json!(true)).unwrap());
    assert!(!support_object_skip_flush(json!(false)).unwrap());
}

#[test]
fn support_object_skip_flush_rejects_non_booleans() {
    for value in [
        json!("true"),
        json!("false"),
        json!(1),
        json!(0),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = support_object_skip_flush(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_object_skip_flush"));
    }
}
