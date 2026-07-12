use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn skirt_start_angle_defaults_to_orca_default() {
    let options = SliceOptions::default();

    assert_eq!(
        options.skirt_options().unwrap().skirt_start_angle_degrees(),
        -135.0
    );
}

#[test]
fn parses_numeric_skirt_start_angle_values() {
    for (value, expected) in [
        (json!(-180), -180.0),
        (json!("-135"), -135.0),
        (json!(0), 0.0),
        (json!("45.5"), 45.5),
        (json!(180), 180.0),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "skirt_start_angle": value })).unwrap();

        assert_eq!(
            options.skirt_options().unwrap().skirt_start_angle_degrees(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_skirt_start_angle_values() {
    for value in [
        json!(-180.1),
        json!(180.1),
        json!("NaN"),
        json!("inf"),
        json!("Infinity"),
        json!("broken"),
        json!(true),
        json!(null),
        json!([]),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "skirt_start_angle": value })).unwrap();

        assert!(matches!(
            options.skirt_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
