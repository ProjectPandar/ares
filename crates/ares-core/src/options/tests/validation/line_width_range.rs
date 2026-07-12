use crate::{SliceError, SliceOptions};
use serde_json::json;

const LINE_WIDTH_KEYS: &[&str] = &[
    "line_width",
    "outer_wall_line_width",
    "inner_wall_line_width",
    "sparse_infill_line_width",
    "internal_solid_infill_line_width",
    "top_surface_line_width",
    "support_line_width",
    "initial_layer_line_width",
];

fn range_message(value: &str) -> String {
    format!("{value} not in range [0.000000,1000.000000]")
}

fn min_only_range_message(value: &str) -> String {
    format!("{value} not in range [0.000000,{:.6}]", f32::MAX as f64)
}

#[test]
fn default_line_width_ranges_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn negative_line_width_values_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": -0.00001,
        "outer_wall_line_width": "-1%",
        "inner_wall_line_width": -2,
        "sparse_infill_line_width": "-3",
        "internal_solid_infill_line_width": -4.5,
        "top_surface_line_width": "-5.5%",
        "support_line_width": -6,
        "initial_layer_line_width": "-7%",
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert_eq!(errors["line_width"], range_message("-0.00001"));
    assert_eq!(errors["outer_wall_line_width"], range_message("-1%"));
    assert_eq!(errors["inner_wall_line_width"], range_message("-2"));
    assert_eq!(errors["sparse_infill_line_width"], range_message("-3"));
    assert_eq!(
        errors["internal_solid_infill_line_width"],
        range_message("-4.5")
    );
    assert_eq!(errors["top_surface_line_width"], range_message("-5.5%"));
    assert_eq!(errors["support_line_width"], range_message("-6"));
    assert_eq!(errors["initial_layer_line_width"], range_message("-7%"));
    assert_eq!(errors.len(), LINE_WIDTH_KEYS.len());
}

#[test]
fn over_max_line_width_values_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 1000.0002,
        "outer_wall_line_width": "1001%",
        "inner_wall_line_width": 1002,
        "sparse_infill_line_width": "1003",
        "internal_solid_infill_line_width": 1004.5,
        "top_surface_line_width": "1005.5%",
        "support_line_width": 1006,
        "initial_layer_line_width": "1007%",
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert_eq!(errors["line_width"], range_message("1000.0002"));
    assert_eq!(errors["outer_wall_line_width"], range_message("1001%"));
    assert_eq!(errors["inner_wall_line_width"], range_message("1002"));
    assert_eq!(errors["sparse_infill_line_width"], range_message("1003"));
    assert_eq!(
        errors["internal_solid_infill_line_width"],
        range_message("1004.5")
    );
    assert_eq!(errors["top_surface_line_width"], range_message("1005.5%"));
    assert_eq!(errors["support_line_width"], range_message("1006"));
    assert_eq!(errors["initial_layer_line_width"], range_message("1007%"));
    assert_eq!(errors.len(), LINE_WIDTH_KEYS.len());
}

#[test]
fn percent_line_width_values_are_range_checked_by_raw_value() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": "1000%",
        "inner_wall_line_width": "1000.0002%",
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(!errors.contains_key("outer_wall_line_width"));
    assert_eq!(errors["inner_wall_line_width"], range_message("1000.0002%"));
    assert_eq!(errors.len(), 1);
}

#[test]
fn line_width_range_boundaries_are_inclusive() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0,
        "outer_wall_line_width": 1000,
        "inner_wall_line_width": "0%",
        "sparse_infill_line_width": "1000%",
        "internal_solid_infill_line_width": "0",
        "top_surface_line_width": "1000",
        "support_line_width": 0.0,
        "initial_layer_line_width": "1000.0%",
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn line_width_range_uses_source_boundary_epsilon() {
    let near_boundary: SliceOptions = serde_json::from_value(json!({
        "outer_wall_line_width": 1000.00001,
        "inner_wall_line_width": "1000.00009%"
    }))
    .unwrap();
    let outside_boundary: SliceOptions = serde_json::from_value(json!({
        "outer_wall_line_width": 1000.00021,
        "inner_wall_line_width": "1000.0002%"
    }))
    .unwrap();

    let near_errors = near_boundary.validate_line_width_range_options().unwrap();
    let outside_errors = outside_boundary
        .validate_line_width_range_options()
        .unwrap();

    assert!(near_errors.is_empty());
    assert_eq!(
        outside_errors["outer_wall_line_width"],
        range_message("1000.00021")
    );
    assert_eq!(
        outside_errors["inner_wall_line_width"],
        range_message("1000.0002%")
    );
}

#[test]
fn invalid_line_width_range_values_return_invalid_input() {
    for (key, value) in [
        ("line_width", json!(true)),
        ("outer_wall_line_width", json!("wide")),
        ("inner_wall_line_width", json!("nan")),
        ("sparse_infill_line_width", json!("nan%")),
        ("internal_solid_infill_line_width", json!({"width": 1})),
        ("top_surface_line_width", json!([1])),
        ("support_line_width", json!("inf")),
        ("initial_layer_line_width", json!("1%%")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options.validate_line_width_range_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)), "{key}");
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_line_width_range_validation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": -0.1,
        "outer_wall_line_width": 2.1,
        "nozzle_diameter": 0.4,
    }))
    .unwrap();

    let range_errors = options.validate_line_width_range_options().unwrap();
    let width_errors = options.validate_extrusion_width_options().unwrap();

    assert_eq!(range_errors["line_width"], range_message("-0.1"));
    assert_eq!(
        width_errors["outer_wall_line_width"],
        "too large line width 2.100000"
    );
}

#[test]
fn min_only_line_width_defaults_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(!errors.contains_key("skin_infill_line_width"));
    assert!(!errors.contains_key("skeleton_infill_line_width"));
}

#[test]
fn min_only_line_width_negative_values_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skin_infill_line_width": -0.00001,
        "skeleton_infill_line_width": "-1%",
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert_eq!(
        errors["skin_infill_line_width"],
        min_only_range_message("-0.00001")
    );
    assert_eq!(
        errors["skeleton_infill_line_width"],
        min_only_range_message("-1%")
    );
    assert_eq!(errors.len(), 2);
}

#[test]
fn min_only_line_width_over_flt_max_values_are_reported() {
    let over_flt_max = (f32::MAX as f64) * 2.0;
    let over_flt_max_json = json!(over_flt_max);
    let over_flt_max_serialized = over_flt_max_json.as_number().unwrap().to_string();
    let over_flt_max_percent = format!("{}%", (f32::MAX as f64) * 3.0);
    let options: SliceOptions = serde_json::from_value(json!({
        "skin_infill_line_width": over_flt_max_json,
        "skeleton_infill_line_width": over_flt_max_percent,
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert_eq!(
        errors["skin_infill_line_width"],
        min_only_range_message(&over_flt_max_serialized)
    );
    assert_eq!(
        errors["skeleton_infill_line_width"],
        min_only_range_message(&over_flt_max_percent)
    );
    assert_eq!(errors.len(), 2);
}

#[test]
fn min_only_line_width_percent_values_use_raw_range() {
    let max_percent = format!("{}%", f32::MAX as f64);
    let over_percent = format!("{}%", (f32::MAX as f64) * 2.0);
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "skin_infill_line_width": max_percent,
        "skeleton_infill_line_width": over_percent,
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(!errors.contains_key("skin_infill_line_width"));
    assert_eq!(
        errors["skeleton_infill_line_width"],
        min_only_range_message(&over_percent)
    );
    assert_eq!(errors.len(), 1);
}

#[test]
fn min_only_line_width_uses_source_boundary_epsilon() {
    let next_after_max = f64::from_bits((f32::MAX as f64).to_bits() + 1);
    let next_after_max_json = json!(next_after_max);
    let next_after_max_serialized = next_after_max_json.as_number().unwrap().to_string();
    let options: SliceOptions = serde_json::from_value(json!({
        "skin_infill_line_width": f32::MAX as f64,
        "skeleton_infill_line_width": next_after_max_json,
    }))
    .unwrap();

    let errors = options.validate_line_width_range_options().unwrap();

    assert!(!errors.contains_key("skin_infill_line_width"));
    assert_eq!(
        errors["skeleton_infill_line_width"],
        min_only_range_message(&next_after_max_serialized)
    );
    assert_eq!(errors.len(), 1);
}

#[test]
fn invalid_min_only_line_width_values_return_invalid_input() {
    for (key, value) in [
        ("skin_infill_line_width", json!(true)),
        ("skeleton_infill_line_width", json!("wide")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options.validate_line_width_range_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)), "{key}");
    }
}
