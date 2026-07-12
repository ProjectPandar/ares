use super::super::timelapse_type::{TimelapseType, TimelapseTypeOptions};
use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn timelapse_type_options(extra: Value) -> Result<TimelapseTypeOptions, SliceError> {
    options(extra).timelapse_type_options()
}

#[test]
fn timelapse_type_defaults_to_traditional() {
    let timelapse_type = SliceOptions::default().timelapse_type_options().unwrap();

    assert_eq!(timelapse_type.mode(), TimelapseType::Traditional);
}

#[test]
fn timelapse_type_accepts_orca_enum_strings() {
    assert_eq!(
        timelapse_type_options(json!({ "timelapse_type": "0" }))
            .unwrap()
            .mode(),
        TimelapseType::Traditional
    );
    assert_eq!(
        timelapse_type_options(json!({ "timelapse_type": "1" }))
            .unwrap()
            .mode(),
        TimelapseType::Smooth
    );
}

#[test]
fn timelapse_type_accepts_direct_legacy_two_after_deserialization() {
    let options: SliceOptions = serde_json::from_value(json!({ "timelapse_type": "2" })).unwrap();

    assert_eq!(options.values()["timelapse_type"], json!("0"));
    assert_eq!(
        options.timelapse_type_options().unwrap().mode(),
        TimelapseType::Traditional
    );
}

#[test]
fn timelapse_type_accepts_legacy_alias_two_after_key_rename() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "timelapse_no_toolhead": "2" })).unwrap();

    assert_eq!(options.values()["timelapse_type"], json!("2"));
    assert_eq!(
        options.timelapse_type_options().unwrap().mode(),
        TimelapseType::Traditional
    );
}

#[test]
fn timelapse_type_rejects_invalid_values() {
    for value in [
        json!(0),
        json!(1),
        json!(2),
        json!("3"),
        json!("traditional"),
        json!("smooth"),
        json!(true),
        json!([]),
        json!({ "value": "0" }),
        Value::Null,
    ] {
        let err = timelapse_type_options(json!({ "timelapse_type": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("timelapse_type"));
    }
}

#[test]
fn timelapse_type_options_can_be_consumed_as_runtime_state() {
    SliceOptions::default()
        .timelapse_type_options()
        .unwrap()
        .consume_runtime();
}
