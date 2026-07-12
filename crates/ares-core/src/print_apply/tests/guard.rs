use serde_json::json;

use super::super::printable_filament_change_guard;
use crate::{Point2, SliceError};

fn map(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .expect("test value must be object")
        .clone()
}

fn square(offset: f64) -> Vec<Point2> {
    vec![
        Point2::new(offset, 0.0),
        Point2::new(offset + 1.0, 0.0),
        Point2::new(offset + 1.0, 1.0),
        Point2::new(offset, 1.0),
    ]
}

#[test]
fn printable_filament_change_equal_polygons_return_false_without_mode() {
    let config = serde_json::Map::new();
    let old_poly = square(0.0);

    let changed = printable_filament_change_guard(&config, &old_poly, &old_poly)
        .expect("equal polygons should not inspect mode");

    assert!(!changed);
}

#[test]
fn printable_filament_change_different_polygons_missing_mode_enters_deferred_branch() {
    let config = serde_json::Map::new();

    let changed = printable_filament_change_guard(&config, &square(0.0), &square(1.0))
        .expect("missing mode should enter deferred geometry branch");

    assert!(changed);
}

#[test]
fn printable_filament_change_manual_mode_returns_false_for_different_polygons() {
    let config = map(json!({ "filament_map_mode": "fmmManual" }));

    let changed = printable_filament_change_guard(&config, &square(0.0), &square(1.0))
        .expect("manual mode should suppress changed printable filament");

    assert!(!changed);
}

#[test]
fn printable_filament_change_legacy_manual_mode_returns_false() {
    let config = map(json!({ "filament_map_mode": "Manual" }));

    let changed = printable_filament_change_guard(&config, &square(0.0), &square(1.0))
        .expect("legacy manual mode should suppress changed printable filament");

    assert!(!changed);
}

#[test]
fn printable_filament_change_non_manual_mode_enters_deferred_branch() {
    let config = map(json!({ "filament_map_mode": "Auto For Flush" }));

    let changed = printable_filament_change_guard(&config, &square(0.0), &square(1.0))
        .expect("non-manual mode should enter deferred geometry branch");

    assert!(changed);
}

#[test]
fn printable_filament_change_rejects_non_string_mode() {
    let config = map(json!({ "filament_map_mode": 2 }));

    let error = printable_filament_change_guard(&config, &square(0.0), &square(1.0))
        .expect_err("present mode must be string");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "filament_map_mode must be a string")
    );
}

use super::super::printable_area_polygons;

#[test]
fn printable_area_polygons_preserve_printable_point_order() {
    let config = map(json!({
        "printable_area": [[0, 0], [200, 0], [200, 200], [0, 200]]
    }));

    let polygons = printable_area_polygons(&config).expect("printable area should parse");

    assert_eq!(
        polygons.printable,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(200.0, 0.0),
            Point2::new(200.0, 200.0),
            Point2::new(0.0, 200.0),
        ]
    );
    assert!(polygons.extruders.is_empty());
}

#[test]
fn printable_area_polygons_preserve_extruder_group_order() {
    let config = map(json!({
        "printable_area": [[0, 0], [10, 0], [10, 10]],
        "extruder_printable_area": [
            [[1, 1], [2, 1], [2, 2]],
            [[3, 3], [4, 3], [4, 4]]
        ]
    }));

    let polygons = printable_area_polygons(&config).expect("extruder areas should parse");

    assert_eq!(polygons.extruders.len(), 2);
    assert_eq!(
        polygons.extruders[0],
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(2.0, 1.0),
            Point2::new(2.0, 2.0),
        ]
    );
    assert_eq!(
        polygons.extruders[1],
        vec![
            Point2::new(3.0, 3.0),
            Point2::new(4.0, 3.0),
            Point2::new(4.0, 4.0),
        ]
    );
}

#[test]
fn printable_area_polygons_require_printable_area() {
    let config = serde_json::Map::new();

    let error = printable_area_polygons(&config).expect_err("printable area is required");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "printable_area must be an array of [x,y] points")
    );
}

#[test]
fn printable_area_polygons_reject_invalid_printable_point() {
    let config = map(json!({ "printable_area": [[0, 0], [1]] }));

    let error = printable_area_polygons(&config).expect_err("invalid point should fail");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "printable_area must be an array of [x,y] points")
    );
}

#[test]
fn printable_area_polygons_reject_invalid_extruder_group() {
    let config = map(json!({
        "printable_area": [[0, 0], [10, 0], [10, 10]],
        "extruder_printable_area": [[[1, 1], [2]]]
    }));

    let error = printable_area_polygons(&config).expect_err("invalid extruder group should fail");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "extruder_printable_area must be an array of point arrays")
    );
}
