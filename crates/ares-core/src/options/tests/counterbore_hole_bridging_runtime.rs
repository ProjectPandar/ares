use super::super::*;
use serde_json::json;

#[test]
fn counterbore_hole_bridging_defaults_to_none() {
    let bridge = SliceOptions::default().bridge_options().unwrap();

    assert_eq!(bridge.counterbore_hole_bridging_for_tests(), "none");
}

#[test]
fn parses_counterbore_hole_bridging_values() {
    for (value, expected) in [
        ("none", "none"),
        ("partiallybridge", "partiallybridge"),
        ("sacrificiallayer", "sacrificiallayer"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "counterbore_hole_bridging": value })).unwrap();

        assert_eq!(
            options
                .bridge_options()
                .unwrap()
                .counterbore_hole_bridging_for_tests(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_counterbore_hole_bridging_values() {
    for value in [json!("bridge"), json!(false)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "counterbore_hole_bridging": value })).unwrap();

        let err = options.bridge_options().unwrap_err();

        assert!(err.to_string().contains("counterbore_hole_bridging"));
    }
}
