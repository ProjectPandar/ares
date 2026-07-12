use super::super::SliceOptions;
use crate::{PrintPathRole, SliceError};
use serde_json::json;

#[test]
fn parses_inner_wall_line_width_for_internal_perimeter_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.5,
        "outer_wall_line_width": 0.3,
        "inner_wall_line_width": "80%"
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();
    let perimeter = options.perimeter_options().unwrap();

    assert!((extrusion.width_for_role(PrintPathRole::InternalPerimeter) - 0.32).abs() < 1e-12);
    assert_eq!(perimeter.external_line_width(), 0.3);
    assert!((perimeter.internal_line_width() - 0.32).abs() < 1e-12);
}

#[test]
fn rejects_invalid_inner_wall_line_width() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "inner_wall_line_width": -0.1 })).unwrap();

    assert!(matches!(
        options.extrusion_options(),
        Err(SliceError::InvalidInput(_))
    ));
}
