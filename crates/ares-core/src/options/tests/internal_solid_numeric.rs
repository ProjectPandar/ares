use super::super::*;
use crate::PrintPathRole;
use serde_json::json;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn internal_solid_line_width_reaches_solid_infill_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "sparse_infill_line_width": 0.3,
        "internal_solid_infill_line_width": 0.6
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_eq!(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.3);
    assert_eq!(extrusion.width_for_role(PrintPathRole::SolidInfill), 0.6);
}

#[test]
fn internal_solid_flow_ratio_is_gated_and_reaches_solid_infill_only() {
    let gated_off: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "internal_solid_infill_flow_ratio": 0.25
    }))
    .unwrap();
    let gated_on: SliceOptions = serde_json::from_value(json!({
        "set_other_flow_ratios": true,
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "sparse_infill_flow_ratio": 0.75,
        "internal_solid_infill_flow_ratio": 0.25
    }))
    .unwrap();

    let off = gated_off.extrusion_options().unwrap();
    let on = gated_on.extrusion_options().unwrap();
    let off_solid = off
        .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
        .unwrap();
    let on_solid = on
        .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
        .unwrap();
    let on_sparse = on
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();

    assert_eq!(round_6(on_solid), round_6(off_solid * 0.25));
    assert_eq!(round_6(on_sparse), round_6(off_solid * 0.75));
}

#[test]
fn validates_internal_solid_flow_ratio_even_when_gate_is_off() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_solid_infill_flow_ratio": value })).unwrap();

        let err = options.extrusion_options().unwrap_err();

        assert!(err.to_string().contains("internal_solid_infill_flow_ratio"));
    }
}

#[test]
fn internal_solid_speed_reaches_solid_infill_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_speed": 80,
        "internal_solid_infill_speed": 120
    }))
    .unwrap();

    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(crate::ToolpathMoveKind::Print, PrintPathRole::SparseInfill),
        80.0
    );
    assert_eq!(
        speeds.speed_for_role(crate::ToolpathMoveKind::Print, PrintPathRole::SolidInfill),
        120.0
    );
}

#[test]
fn internal_solid_acceleration_reaches_solid_infill_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "default_acceleration": 700,
        "sparse_infill_acceleration": "50%",
        "internal_solid_infill_acceleration": "25%"
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.acceleration_for_layer(
            crate::ToolpathMoveKind::Print,
            PrintPathRole::SparseInfill,
            false
        ),
        Some(350.0)
    );
    assert_eq!(
        speeds.acceleration_for_layer(
            crate::ToolpathMoveKind::Print,
            PrintPathRole::SolidInfill,
            false
        ),
        Some(175.0)
    );
}

#[test]
fn rejects_invalid_internal_solid_speed_and_acceleration_values() {
    for (key, value) in [
        ("internal_solid_infill_speed", json!(0)),
        ("internal_solid_infill_speed", json!(-1)),
        ("internal_solid_infill_speed", json!("NaN")),
        ("internal_solid_infill_acceleration", json!(-1)),
        ("internal_solid_infill_acceleration", json!("bad%")),
        ("internal_solid_infill_acceleration", json!(false)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(err.to_string().contains(key), "{err}");
    }
}
