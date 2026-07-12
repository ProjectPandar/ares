use super::super::*;
use serde_json::json;

#[test]
fn overhang_reverse_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options.perimeter_options().unwrap().overhang_reverse());
}

#[test]
fn parses_overhang_reverse_true() {
    let options: SliceOptions = serde_json::from_value(json!({
        "overhang_reverse": true
    }))
    .unwrap();

    assert!(options.perimeter_options().unwrap().overhang_reverse());
}

#[test]
fn rejects_invalid_overhang_reverse_values() {
    for value in [json!("true"), json!(1), json!(null), json!([])] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "overhang_reverse": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn overhang_reverse_internal_only_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options
        .perimeter_options()
        .unwrap()
        .overhang_reverse_internal_only());
}

#[test]
fn parses_overhang_reverse_internal_only_true() {
    let options: SliceOptions = serde_json::from_value(json!({
        "overhang_reverse_internal_only": true
    }))
    .unwrap();

    assert!(options
        .perimeter_options()
        .unwrap()
        .overhang_reverse_internal_only());
}

#[test]
fn rejects_invalid_overhang_reverse_internal_only_values() {
    for value in [json!("true"), json!(1), json!(null), json!([])] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "overhang_reverse_internal_only": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn overhang_reverse_threshold_defaults_to_half_line_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4
    }))
    .unwrap();

    assert_eq!(
        options
            .perimeter_options()
            .unwrap()
            .overhang_reverse_threshold_mm(),
        0.2
    );
}

#[test]
fn parses_overhang_reverse_threshold_percent_over_external_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "overhang_reverse_threshold": "25%"
    }))
    .unwrap();

    assert_eq!(
        options
            .perimeter_options()
            .unwrap()
            .overhang_reverse_threshold_mm(),
        0.1
    );
}

#[test]
fn accepts_overhang_reverse_threshold_numeric_bounds() {
    for value in [json!(0), json!(20), json!("0"), json!("20")] {
        let options: SliceOptions = serde_json::from_value(json!({
            "overhang_reverse_threshold": value
        }))
        .unwrap();

        assert!(options.perimeter_options().is_ok());
    }
}

#[test]
fn rejects_invalid_overhang_reverse_threshold_values() {
    for value in [
        json!(-0.1),
        json!(20.1),
        json!("20.1"),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("NaN%"),
        json!("inf%"),
        json!("-inf%"),
        json!(true),
        json!(null),
        json!([]),
        json!({}),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "overhang_reverse_threshold": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
