use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn min_skirt_length_defaults_to_zero() {
    let options = SliceOptions::default();

    assert_eq!(
        options.skirt_options().unwrap().min_skirt_length_mm(),
        0.0
    );
}

#[test]
fn parses_numeric_min_skirt_length_values() {
    for value in [json!(12.5), json!("12.5"), json!(0), json!("0")] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "min_skirt_length": value })).unwrap();

        assert!(options.skirt_options().unwrap().min_skirt_length_mm() >= 0.0);
    }
}

#[test]
fn rejects_invalid_min_skirt_length_values() {
    for value in [
        json!(-0.1),
        json!("broken"),
        json!(true),
        json!(null),
        json!(["12"]),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "min_skirt_length": value })).unwrap();

        assert!(matches!(
            options.skirt_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
