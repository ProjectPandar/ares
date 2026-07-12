use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn placement(
    key: &str,
    value: Value,
) -> Result<super::super::support_placement::SupportPlacementOptions, SliceError> {
    options(json!({ key: value })).support_placement_options()
}

fn float_value(key: &str, value: Value) -> Result<f64, SliceError> {
    let placement = placement(key, value)?;
    Ok(match key {
        "support_object_xy_distance" => placement.object_xy_distance_mm(),
        "support_object_first_layer_gap" => placement.object_first_layer_gap_mm(),
        _ => unreachable!("test only passes known support placement float keys"),
    })
}

fn bool_value(key: &str, value: Value) -> Result<bool, SliceError> {
    let placement = placement(key, value)?;
    Ok(match key {
        "support_on_build_plate_only" => placement.on_build_plate_only(),
        "support_critical_regions_only" => placement.critical_regions_only(),
        "support_remove_small_overhang" => placement.remove_small_overhang(),
        _ => unreachable!("test only passes known support placement bool keys"),
    })
}

#[test]
fn support_placement_options_default_to_orca_values() {
    let placement = SliceOptions::default().support_placement_options().unwrap();

    assert_eq!(placement.object_xy_distance_mm(), 0.35);
    assert_eq!(placement.object_first_layer_gap_mm(), 0.2);
    assert!(!placement.on_build_plate_only());
    assert!(!placement.critical_regions_only());
    assert!(placement.remove_small_overhang());
}

#[test]
fn support_placement_options_parse_numeric_values_strings_and_bools() {
    let placement = options(json!({
        "support_object_xy_distance": "1.25",
        "support_object_first_layer_gap": 0.75,
        "support_on_build_plate_only": true,
        "support_critical_regions_only": true,
        "support_remove_small_overhang": false
    }))
    .support_placement_options()
    .unwrap();

    assert_eq!(placement.object_xy_distance_mm(), 1.25);
    assert_eq!(placement.object_first_layer_gap_mm(), 0.75);
    assert!(placement.on_build_plate_only());
    assert!(placement.critical_regions_only());
    assert!(!placement.remove_small_overhang());
}

#[test]
fn support_placement_float_options_accept_orca_boundaries() {
    for key in [
        "support_object_xy_distance",
        "support_object_first_layer_gap",
    ] {
        assert_eq!(float_value(key, json!(0.0)).unwrap(), 0.0);
        assert_eq!(float_value(key, json!("0.0")).unwrap(), 0.0);
        assert_eq!(float_value(key, json!(4.25)).unwrap(), 4.25);
        assert_eq!(float_value(key, json!("4.25")).unwrap(), 4.25);
        assert_eq!(float_value(key, json!(10.0)).unwrap(), 10.0);
        assert_eq!(float_value(key, json!("10.0")).unwrap(), 10.0);
    }
}

#[test]
fn support_placement_bool_options_accept_booleans() {
    for key in [
        "support_on_build_plate_only",
        "support_critical_regions_only",
        "support_remove_small_overhang",
    ] {
        assert!(bool_value(key, json!(true)).unwrap());
        assert!(!bool_value(key, json!(false)).unwrap());
    }
}

#[test]
fn support_placement_float_options_reject_invalid_values() {
    for key in [
        "support_object_xy_distance",
        "support_object_first_layer_gap",
    ] {
        for value in [
            json!(-0.001),
            json!(10.001),
            json!("NaN"),
            json!("inf"),
            json!("-inf"),
            json!("invalid"),
            json!(true),
            json!(false),
            json!([]),
            json!({}),
            Value::Null,
        ] {
            let err = float_value(key, value).unwrap_err();
            assert!(matches!(err, SliceError::InvalidInput(_)));
            assert!(err.to_string().contains(key));
        }
    }
}

#[test]
fn support_placement_bool_options_reject_non_booleans() {
    for key in [
        "support_on_build_plate_only",
        "support_critical_regions_only",
        "support_remove_small_overhang",
    ] {
        for value in [
            json!("true"),
            json!("false"),
            json!(1),
            json!(0),
            json!([]),
            json!({}),
            Value::Null,
        ] {
            let err = bool_value(key, value).unwrap_err();
            assert!(matches!(err, SliceError::InvalidInput(_)));
            assert!(err.to_string().contains(key));
        }
    }
}

#[test]
fn legacy_percentage_support_object_xy_distance_resolves_to_default() {
    let options: SliceOptions = serde_json::from_value(json!({
        "support_object_xy_distance": "80%"
    }))
    .unwrap();

    assert!(!options.values().contains_key("support_object_xy_distance"));
    assert_eq!(
        options
            .support_placement_options()
            .unwrap()
            .object_xy_distance_mm(),
        0.35
    );
}

#[test]
fn obsolete_support_remove_small_overhangs_key_does_not_override_default() {
    let options: SliceOptions = serde_json::from_value(json!({
        "support_remove_small_overhangs": false
    }))
    .unwrap();

    assert!(!options.values().contains_key("support_remove_small_overhangs"));
    assert!(
        options
            .support_placement_options()
            .unwrap()
            .remove_small_overhang()
    );
}
