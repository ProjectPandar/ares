use super::super::*;
use serde_json::json;

#[test]
fn internal_bridge_density_defaults_to_full_internal_bridge() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.internal_bridge_density_percent(), 100.0);
}

#[test]
fn parses_internal_bridge_density_runtime_values() {
    for (value, expected) in [(json!(10), 10.0), (json!(100), 100.0), (json!("75"), 75.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_density": value })).unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .internal_bridge_density_percent(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_internal_bridge_density_values() {
    for value in [
        json!(9.9),
        json!(100.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_density": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("internal_bridge_density"), "{err}");
    }
}
