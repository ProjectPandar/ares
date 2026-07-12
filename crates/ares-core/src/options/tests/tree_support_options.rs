use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn parsed_float(key: &str, value: Value) -> Result<f64, SliceError> {
    let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
    let tree = options.tree_support_options()?;
    Ok(match key {
        "tree_support_branch_distance" => tree.branch_distance_mm(),
        "tree_support_tip_diameter" => tree.tip_diameter_mm(),
        "tree_support_branch_diameter" => tree.branch_diameter_mm(),
        "tree_support_branch_angle" => tree.branch_angle_degrees(),
        "tree_support_branch_diameter_angle" => tree.branch_diameter_angle_degrees(),
        "tree_support_angle_slow" => tree.angle_slow_degrees(),
        "tree_support_brim_width" => tree.brim_width_mm(),
        "tree_support_branch_distance_organic" => tree.branch_distance_organic_mm(),
        "tree_support_top_rate" => tree.top_rate_percent(),
        "tree_support_branch_diameter_organic" => tree.branch_diameter_organic_mm(),
        "tree_support_branch_angle_organic" => tree.branch_angle_organic_degrees(),
        _ => unreachable!("test only passes known tree support float option keys"),
    })
}

fn parsed_wall_count(value: Value) -> Result<u32, SliceError> {
    let options: SliceOptions =
        serde_json::from_value(json!({ "tree_support_wall_count": value })).unwrap();
    Ok(options.tree_support_options()?.wall_count())
}

fn parsed_auto_brim(value: Value) -> Result<bool, SliceError> {
    let options: SliceOptions =
        serde_json::from_value(json!({ "tree_support_auto_brim": value })).unwrap();
    Ok(options.tree_support_options()?.auto_brim())
}

#[test]
fn tree_support_options_default_to_orca_values() {
    let tree = SliceOptions::default().tree_support_options().unwrap();

    assert_eq!(tree.branch_distance_mm(), 5.0);
    assert_eq!(tree.tip_diameter_mm(), 0.8);
    assert_eq!(tree.branch_diameter_mm(), 5.0);
    assert_eq!(tree.branch_angle_degrees(), 40.0);
    assert_eq!(tree.branch_diameter_angle_degrees(), 5.0);
    assert_eq!(tree.angle_slow_degrees(), 25.0);
    assert_eq!(tree.wall_count(), 0);
    assert!(tree.auto_brim());
    assert_eq!(tree.brim_width_mm(), 3.0);
    assert_eq!(tree.branch_distance_organic_mm(), 1.0);
    assert_eq!(tree.top_rate_percent(), 30.0);
    assert_eq!(tree.branch_diameter_organic_mm(), 2.0);
    assert_eq!(tree.branch_angle_organic_degrees(), 40.0);
}

#[test]
fn parses_tree_support_numeric_values_strings_and_bools() {
    let options: SliceOptions = serde_json::from_value(json!({
        "tree_support_branch_distance": "6.5",
        "tree_support_tip_diameter": 0.9,
        "tree_support_branch_diameter": "4",
        "tree_support_branch_angle": 55.5,
        "tree_support_branch_diameter_angle": "7.5",
        "tree_support_angle_slow": 30,
        "tree_support_wall_count": "2",
        "tree_support_auto_brim": false,
        "tree_support_brim_width": "4.25",
        "tree_support_branch_distance_organic": "3.5",
        "tree_support_top_rate": 12,
        "tree_support_branch_diameter_organic": "4",
        "tree_support_branch_angle_organic": 55.5
    }))
    .unwrap();

    let tree = options.tree_support_options().unwrap();

    assert_eq!(tree.branch_distance_mm(), 6.5);
    assert_eq!(tree.tip_diameter_mm(), 0.9);
    assert_eq!(tree.branch_diameter_mm(), 4.0);
    assert_eq!(tree.branch_angle_degrees(), 55.5);
    assert_eq!(tree.branch_diameter_angle_degrees(), 7.5);
    assert_eq!(tree.angle_slow_degrees(), 30.0);
    assert_eq!(tree.wall_count(), 2);
    assert!(!tree.auto_brim());
    assert_eq!(tree.brim_width_mm(), 4.25);
    assert_eq!(tree.branch_distance_organic_mm(), 3.5);
    assert_eq!(tree.top_rate_percent(), 12.0);
    assert_eq!(tree.branch_diameter_organic_mm(), 4.0);
    assert_eq!(tree.branch_angle_organic_degrees(), 55.5);
}

#[test]
fn tree_support_float_options_accept_orca_boundaries() {
    for (key, low, high) in [
        ("tree_support_branch_distance", 1.0, 10.0),
        ("tree_support_tip_diameter", 0.1, 100.0),
        ("tree_support_branch_diameter", 1.0, 10.0),
        ("tree_support_branch_angle", 0.0, 60.0),
        ("tree_support_branch_diameter_angle", 0.0, 15.0),
        ("tree_support_angle_slow", 10.0, 85.0),
        ("tree_support_branch_distance_organic", 1.0, 10.0),
        ("tree_support_top_rate", 5.0, 35.0),
        ("tree_support_branch_diameter_organic", 1.0, 10.0),
        ("tree_support_branch_angle_organic", 0.0, 60.0),
    ] {
        assert_eq!(parsed_float(key, json!(low)).unwrap(), low);
        assert_eq!(parsed_float(key, json!(high)).unwrap(), high);
    }

    assert_eq!(
        parsed_float("tree_support_brim_width", json!(0.0)).unwrap(),
        0.0
    );
    assert_eq!(
        parsed_float("tree_support_brim_width", json!("12345.5")).unwrap(),
        12345.5
    );
}

#[test]
fn tree_support_wall_count_accepts_orca_boundaries() {
    assert_eq!(parsed_wall_count(json!(0)).unwrap(), 0);
    assert_eq!(parsed_wall_count(json!("0")).unwrap(), 0);
    assert_eq!(parsed_wall_count(json!(2)).unwrap(), 2);
    assert_eq!(parsed_wall_count(json!("2")).unwrap(), 2);
}

#[test]
fn tree_support_auto_brim_accepts_booleans() {
    assert!(parsed_auto_brim(json!(true)).unwrap());
    assert!(!parsed_auto_brim(json!(false)).unwrap());
}

#[test]
fn tree_support_float_options_reject_invalid_values() {
    for (key, below, above) in [
        ("tree_support_branch_distance", 0.999, 10.001),
        ("tree_support_tip_diameter", 0.099, 100.001),
        ("tree_support_branch_diameter", 0.999, 10.001),
        ("tree_support_branch_angle", -0.001, 60.001),
        ("tree_support_branch_diameter_angle", -0.001, 15.001),
        ("tree_support_angle_slow", 9.999, 85.001),
        ("tree_support_branch_distance_organic", 0.999, 10.001),
        ("tree_support_top_rate", 4.999, 35.001),
        ("tree_support_branch_diameter_organic", 0.999, 10.001),
        ("tree_support_branch_angle_organic", -0.001, 60.001),
    ] {
        for value in [
            json!(below),
            json!(above),
            json!("NaN"),
            json!("inf"),
            json!("-inf"),
            json!("invalid"),
            json!(true),
            json!([]),
            json!({}),
            Value::Null,
        ] {
            let err = parsed_float(key, value).unwrap_err();
            assert!(matches!(err, SliceError::InvalidInput(_)));
            assert!(err.to_string().contains(key));
        }
    }
}

#[test]
fn tree_support_brim_width_rejects_invalid_values() {
    for value in [
        json!(-0.001),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("invalid"),
        json!(true),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = parsed_float("tree_support_brim_width", value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("tree_support_brim_width"));
    }
}

#[test]
fn tree_support_wall_count_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!(3),
        json!(1.0),
        json!(1.5),
        json!("3"),
        json!("1.0"),
        json!("1.5"),
        json!("invalid"),
        json!(true),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = parsed_wall_count(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("tree_support_wall_count"));
    }
}

#[test]
fn tree_support_auto_brim_rejects_non_booleans() {
    for value in [
        json!(1),
        json!("true"),
        json!("false"),
        json!("invalid"),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = parsed_auto_brim(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("tree_support_auto_brim"));
    }
}
