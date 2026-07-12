use super::super::*;
use crate::{SkirtType, SliceError};
use serde_json::json;

#[test]
fn skirt_type_defaults_to_combined() {
    let options = SliceOptions::default();

    assert_eq!(
        options.skirt_options().unwrap().skirt_type(),
        SkirtType::Combined
    );
}

#[test]
fn parses_supported_skirt_type_values() {
    for (value, expected) in [
        ("combined", SkirtType::Combined),
        ("perobject", SkirtType::PerObject),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "skirt_type": value })).unwrap();

        assert_eq!(options.skirt_options().unwrap().skirt_type(), expected);
    }
}

#[test]
fn rejects_invalid_skirt_type_values() {
    for value in [
        json!("per-object"),
        json!("none"),
        json!(true),
        json!(1),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "skirt_type": value })).unwrap();

        assert!(matches!(
            options.skirt_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
