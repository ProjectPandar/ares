use super::super::*;
use serde_json::json;

#[test]
fn accel_to_decel_defaults_match_orca() {
    let config = SliceOptions::default().accel_to_decel_config().unwrap();

    assert!(config.enabled());
    assert_eq!(config.factor_percent(), 50.0);
}

#[test]
fn accel_to_decel_parses_explicit_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "accel_to_decel_enable": false,
        "accel_to_decel_factor": 33
    }))
    .unwrap();

    let config = options.accel_to_decel_config().unwrap();

    assert!(!config.enabled());
    assert_eq!(config.factor_percent(), 33.0);
}

#[test]
fn accel_to_decel_preserves_decimal_factor_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "accel_to_decel_factor": 33.5
    }))
    .unwrap();

    let config = options.accel_to_decel_config().unwrap();

    assert_eq!(config.factor_percent(), 33.5);
}

#[test]
fn accel_to_decel_rejects_non_boolean_enable() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "accel_to_decel_enable": "true" })).unwrap();

    let err = options.accel_to_decel_config().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("accel_to_decel_enable must be a boolean")
    );
}

#[test]
fn accel_to_decel_rejects_invalid_factor_values() {
    for value in [
        json!(0),
        json!(101),
        json!(-1),
        json!("33"),
        json!(true),
        json!(null),
        json!([33]),
        json!({ "value": 33 }),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "accel_to_decel_factor": value })).unwrap();

        let err = options.accel_to_decel_config().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("accel_to_decel_factor"));
    }
}

#[test]
fn accel_to_decel_documents_non_finite_json_boundary() {
    let err = serde_json::from_str::<SliceOptions>(r#"{ "accel_to_decel_factor": 1e999 }"#)
        .unwrap_err();

    assert!(err.to_string().contains("number out of range"));
}
