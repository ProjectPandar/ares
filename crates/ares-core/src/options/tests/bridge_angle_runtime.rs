use super::super::*;
use serde_json::json;

#[test]
fn bridge_angle_defaults_to_orca_auto_value() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.bridge_angle_degrees(), 0.0);
}

#[test]
fn parses_bridge_angle_runtime_values() {
    for (value, expected) in [(json!(90), 90.0), (json!("180"), 180.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "bridge_angle": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().bridge_angle_degrees(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_bridge_angle_values() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "bridge_angle": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("bridge_angle"), "{err}");
    }
}
