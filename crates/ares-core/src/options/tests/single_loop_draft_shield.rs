use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn single_loop_draft_shield_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options.skirt_options().unwrap().single_loop_draft_shield());
}

#[test]
fn parses_boolean_single_loop_draft_shield_values() {
    for (value, expected) in [(json!(true), true), (json!(false), false)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "single_loop_draft_shield": value })).unwrap();

        assert_eq!(
            options.skirt_options().unwrap().single_loop_draft_shield(),
            expected
        );
    }
}

#[test]
fn rejects_non_boolean_single_loop_draft_shield_values() {
    for value in [json!("true"), json!(1), json!(null), json!([])] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "single_loop_draft_shield": value })).unwrap();

        assert!(matches!(
            options.skirt_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
