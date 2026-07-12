use super::super::*;
use serde_json::json;

#[test]
fn enable_extra_bridge_layer_defaults_to_disabled() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    assert_eq!(
        options
            .bridge_options()
            .unwrap()
            .extra_bridge_layer_for_tests(),
        "disabled"
    );
}

#[test]
fn parses_enable_extra_bridge_layer_enum_values() {
    for value in [
        "disabled",
        "external_bridge_only",
        "internal_bridge_only",
        "apply_to_all",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "enable_extra_bridge_layer": value
        }))
        .unwrap();

        assert_eq!(
            options
                .bridge_options()
                .unwrap()
                .extra_bridge_layer_for_tests(),
            value
        );
    }
}

#[test]
fn rejects_invalid_enable_extra_bridge_layer_values() {
    for value in [json!("bad"), json!(true), json!(1), json!(["disabled"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "enable_extra_bridge_layer": value
        }))
        .unwrap();

        let err = options.bridge_options().unwrap_err();
        assert!(err.to_string().contains("enable_extra_bridge_layer"));
    }
}
