use super::super::*;
use serde_json::json;

#[test]
fn filter_out_gap_fill_defaults_to_zero() {
    let options = SliceOptions::default();

    assert_eq!(options.filter_out_gap_fill_mm().unwrap(), 0.0);
}

#[test]
fn filter_out_gap_fill_accepts_finite_numbers_and_numeric_strings() {
    let numeric: SliceOptions =
        serde_json::from_value(json!({ "filter_out_gap_fill": 1.25 })).unwrap();
    let string: SliceOptions =
        serde_json::from_value(json!({ "filter_out_gap_fill": "2.5" })).unwrap();
    let negative: SliceOptions =
        serde_json::from_value(json!({ "filter_out_gap_fill": -1.0 })).unwrap();

    assert_eq!(numeric.filter_out_gap_fill_mm().unwrap(), 1.25);
    assert_eq!(string.filter_out_gap_fill_mm().unwrap(), 2.5);
    assert_eq!(negative.filter_out_gap_fill_mm().unwrap(), -1.0);
}

#[test]
fn filter_out_gap_fill_rejects_non_finite_and_non_numeric_values() {
    for value in [
        json!(false),
        json!("wide"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filter_out_gap_fill": value })).unwrap();

        let err = options.filter_out_gap_fill_mm().unwrap_err();

        assert!(err.to_string().contains("filter_out_gap_fill"));
    }
}
