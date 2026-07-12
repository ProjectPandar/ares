use serde_json::json;

use crate::{InfillOptions, InfillPattern, SliceError, SliceOptions};

#[test]
fn sparse_infill_pattern_accepts_zigzag() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_pattern": "zigzag"
    }))
    .unwrap();

    assert_eq!(
        options.infill_options().unwrap().pattern(),
        InfillPattern::ZigZag
    );
}

#[test]
fn sparse_infill_pattern_accepts_crosszag() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_pattern": "crosszag"
    }))
    .unwrap();

    assert_eq!(
        options.infill_options().unwrap().pattern(),
        InfillPattern::CrossZag
    );
}

#[test]
fn sparse_infill_pattern_accepts_lockedzag() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_pattern": "lockedzag"
    }))
    .unwrap();

    assert_eq!(
        options.infill_options().unwrap().pattern(),
        InfillPattern::LockedZag
    );
}

#[test]
fn lockedzag_remains_invalid_for_surface_patterns() {
    for key in [
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        let mut value = serde_json::Map::new();
        value.insert(key.to_owned(), json!("lockedzag"));
        let options: SliceOptions =
            serde_json::from_value(serde_json::Value::Object(value)).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains(key) && message.contains("lockedzag")
        ));
    }
}

#[test]
fn infill_shift_step_defaults_to_orca_value() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.infill_shift_step_mm(), 0.4);
    assert_eq!(
        InfillOptions::new_for_tests(20.0, 45.0, 0.4)
            .with_infill_shift_step_for_tests(0.25)
            .infill_shift_step_mm(),
        0.25
    );
}

#[test]
fn parses_infill_shift_step_values() {
    for (value, expected) in [(json!(0), 0.0), (json!(0.25), 0.25), (json!("10"), 10.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "infill_shift_step": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().infill_shift_step_mm(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_infill_shift_step_values() {
    for value in [
        json!(-0.1),
        json!(10.1),
        json!("NaN"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "infill_shift_step": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message)) if message.contains("infill_shift_step")
        ));
    }
}
