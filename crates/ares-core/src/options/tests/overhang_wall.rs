use super::super::*;
use serde_json::json;

#[test]
fn rejects_invalid_detect_overhang_wall() {
    let options: SliceOptions = serde_json::from_value(json!({
        "detect_overhang_wall": "yes"
    }))
    .unwrap();

    assert!(matches!(
        options.perimeter_options(),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn extra_perimeters_on_overhangs_defaults_false_parses_bool_and_rejects_invalid() {
    let default = SliceOptions::default().perimeter_options().unwrap();
    assert!(!default.extra_perimeters_on_overhangs());

    let enabled: SliceOptions = serde_json::from_value(json!({
        "extra_perimeters_on_overhangs": true
    }))
    .unwrap();
    assert!(
        enabled
            .perimeter_options()
            .unwrap()
            .extra_perimeters_on_overhangs()
    );

    let invalid: SliceOptions = serde_json::from_value(json!({
        "extra_perimeters_on_overhangs": "true"
    }))
    .unwrap();
    let err = invalid.perimeter_options().unwrap_err();
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("extra_perimeters_on_overhangs"));
}

#[test]
fn make_overhang_printable_defaults_and_numeric_options_match_orca() {
    let default = SliceOptions::default().perimeter_options().unwrap();
    assert!(!default.make_overhang_printable());
    assert_eq!(default.make_overhang_printable_angle_degrees(), 55.0);
    assert_eq!(default.make_overhang_printable_hole_size_mm2(), 0.0);

    let enabled: SliceOptions = serde_json::from_value(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": "45",
        "make_overhang_printable_hole_size": "1.5"
    }))
    .unwrap();
    let perimeters = enabled.perimeter_options().unwrap();
    assert!(perimeters.make_overhang_printable());
    assert_eq!(perimeters.make_overhang_printable_angle_degrees(), 45.0);
    assert_eq!(perimeters.make_overhang_printable_hole_size_mm2(), 1.5);
}

#[test]
fn make_overhang_printable_rejects_invalid_option_values() {
    for (key, value) in [
        ("make_overhang_printable", json!("true")),
        ("make_overhang_printable_angle", json!(-0.1)),
        ("make_overhang_printable_angle", json!(90.1)),
        ("make_overhang_printable_angle", json!("NaN")),
        ("make_overhang_printable_angle", json!([])),
        ("make_overhang_printable_hole_size", json!(-0.1)),
        ("make_overhang_printable_hole_size", json!("NaN")),
        ("make_overhang_printable_hole_size", json!({})),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = options.perimeter_options().unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
