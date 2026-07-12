use super::super::*;
use serde_json::json;

#[test]
fn dont_filter_internal_bridges_defaults_to_disabled_filter() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    assert_eq!(
        options
            .infill_options()
            .unwrap()
            .internal_bridge_filter_for_tests(),
        "disabled"
    );
}

#[test]
fn parses_dont_filter_internal_bridges_enum_values() {
    for value in ["disabled", "limited", "nofilter"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "dont_filter_internal_bridges": value
        }))
        .unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .internal_bridge_filter_for_tests(),
            value
        );
    }
}

#[test]
fn rejects_invalid_dont_filter_internal_bridges_values() {
    for value in [json!("bad"), json!(true), json!(1), json!(["limited"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "dont_filter_internal_bridges": value
        }))
        .unwrap();

        let err = options.infill_options().unwrap_err();
        assert!(err.to_string().contains("dont_filter_internal_bridges"));
    }
}
