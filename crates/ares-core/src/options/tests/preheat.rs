use super::super::preheat::PreheatOptions;
use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn preheat_options(extra: Value) -> Result<PreheatOptions, SliceError> {
    options(extra).preheat_options()
}

#[test]
fn preheat_defaults_to_orca_values() {
    let preheat = SliceOptions::default().preheat_options().unwrap();

    assert_eq!(preheat.time_s(), 30.0);
    assert_eq!(preheat.steps(), 1);
}

#[test]
fn preheat_time_accepts_boundaries_and_numeric_strings() {
    assert_eq!(
        preheat_options(json!({ "preheat_time": 0.0 }))
            .unwrap()
            .time_s(),
        0.0
    );
    assert_eq!(
        preheat_options(json!({ "preheat_time": "0" }))
            .unwrap()
            .time_s(),
        0.0
    );
    assert_eq!(
        preheat_options(json!({ "preheat_time": 15.5 }))
            .unwrap()
            .time_s(),
        15.5
    );
    assert_eq!(
        preheat_options(json!({ "preheat_time": "15.5" }))
            .unwrap()
            .time_s(),
        15.5
    );
    assert_eq!(
        preheat_options(json!({ "preheat_time": 120.0 }))
            .unwrap()
            .time_s(),
        120.0
    );
}

#[test]
fn preheat_time_rejects_invalid_values() {
    for value in [
        json!(-0.001),
        json!("-0.001"),
        json!(120.001),
        json!("120.001"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("invalid"),
        json!(true),
        json!([]),
        json!({ "value": 30 }),
        Value::Null,
    ] {
        let err = preheat_options(json!({ "preheat_time": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("preheat_time"));
    }
}

#[test]
fn preheat_steps_accepts_boundaries_and_integer_strings() {
    assert_eq!(
        preheat_options(json!({ "preheat_steps": 1 }))
            .unwrap()
            .steps(),
        1
    );
    assert_eq!(
        preheat_options(json!({ "preheat_steps": "1" }))
            .unwrap()
            .steps(),
        1
    );
    assert_eq!(
        preheat_options(json!({ "preheat_steps": 10 }))
            .unwrap()
            .steps(),
        10
    );
    assert_eq!(
        preheat_options(json!({ "preheat_steps": "10" }))
            .unwrap()
            .steps(),
        10
    );
}

#[test]
fn preheat_steps_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!("-1"),
        json!(0),
        json!("0"),
        json!(11),
        json!("11"),
        json!(1.5),
        json!("1.5"),
        json!("invalid"),
        json!(true),
        json!([]),
        json!({ "value": 1 }),
        Value::Null,
    ] {
        let err = preheat_options(json!({ "preheat_steps": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("preheat_steps"));
    }
}

#[test]
fn preheat_options_can_be_consumed_as_runtime_state() {
    SliceOptions::default()
        .preheat_options()
        .unwrap()
        .consume_runtime();
}
