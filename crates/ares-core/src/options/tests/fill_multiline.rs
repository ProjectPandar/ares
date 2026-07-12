use super::super::*;
use serde_json::json;

#[test]
fn fill_multiline_defaults_match_orca() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.fill_multiline(), 1);
}

#[test]
fn parses_valid_fill_multiline_values() {
    for (value, expected) in [(json!(1), 1), (json!(10), 10), (json!("3"), 3)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fill_multiline": value })).unwrap();

        assert_eq!(options.infill_options().unwrap().fill_multiline(), expected);
    }
}

#[test]
fn rejects_invalid_fill_multiline_values() {
    for value in [
        json!(0),
        json!(-1),
        json!(11),
        json!(1.5),
        json!("1.5"),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!(true),
        json!(null),
        json!([1]),
        json!({"value": 1}),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fill_multiline": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message)) if message.contains("fill_multiline")
        ));
    }
}
