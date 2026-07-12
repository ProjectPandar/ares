use super::super::*;
use crate::{ExtrusionOptions, PrintPathRole};
use serde_json::json;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn rejects_invalid_thick_internal_bridges() {
    let options: SliceOptions = serde_json::from_value(json!({
        "thick_internal_bridges": 1
    }))
    .unwrap();

    assert!(options.bridge_options().is_err());
}

#[test]
fn parsed_thick_internal_bridges_reaches_extrusion_options() {
    let thin_options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0.5,
        "internal_bridge_flow": 0.25,
        "thick_internal_bridges": false,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let thick_options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0.5,
        "internal_bridge_flow": 0.25,
        "thick_internal_bridges": true,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();

    let thin = thin_options
        .extrusion_options()
        .unwrap()
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();
    let thick = thick_options
        .extrusion_options()
        .unwrap()
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();

    assert_ne!(round_6(thin), round_6(thick));
}

#[test]
fn missing_thick_internal_bridges_defaults_to_thick_internal_bridge_extrusion() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    let internal_bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();

    assert_eq!(round_6(internal_bridge_e), round_6(expected));
}

#[test]
fn explicit_false_thick_internal_bridges_preserves_thin_internal_bridge_extrusion() {
    let options: SliceOptions = serde_json::from_value(json!({
        "internal_bridge_flow": 0.25,
        "thick_internal_bridges": false,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    let internal_bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();

    assert_eq!(round_6(internal_bridge_e), round_6(base_e * 0.25));
}
