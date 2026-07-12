use super::super::*;
use serde_json::json;

#[test]
fn internal_bridge_angle_defaults_to_orca_auto_value() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.internal_bridge_angle_degrees(), 0.0);
}

#[test]
fn parses_internal_bridge_angle_runtime_values() {
    for (value, expected) in [(json!(90), 90.0), (json!("180"), 180.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_angle": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().internal_bridge_angle_degrees(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_internal_bridge_angle_values() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_angle": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("internal_bridge_angle"), "{err}");
    }
}
