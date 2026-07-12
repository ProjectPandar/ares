use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn minimum_sparse_infill_area_defaults_to_orca_value() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.minimum_sparse_infill_area_mm2(), 15.0);
}

#[test]
fn parses_minimum_sparse_infill_area_values() {
    for (value, expected) in [(json!(0), 0.0), (json!(20.5), 20.5), (json!("30"), 30.0)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "minimum_sparse_infill_area": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().minimum_sparse_infill_area_mm2(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_minimum_sparse_infill_area_values() {
    for value in [
        json!(-0.1),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!(true),
        json!(null),
        json!([]),
        json!({}),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "minimum_sparse_infill_area": value })).unwrap();
        let err = options.infill_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("minimum_sparse_infill_area"));
    }
}
