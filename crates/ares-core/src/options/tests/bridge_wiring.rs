use super::super::*;
use crate::{ExtrusionOptions, PrintPathRole, ToolpathMoveKind};
use serde_json::json;

#[test]
fn parsed_bridge_flow_reaches_extrusion_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0.5,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

    let bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::Bridge, 0.2)
        .unwrap();
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
}

#[test]
fn parsed_internal_bridge_flow_reaches_extrusion_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0.5,
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
    let bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::Bridge, 0.2)
        .unwrap();
    let internal_bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();

    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (internal_bridge_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
}

#[test]
fn parsed_brim_flow_ratio_reaches_extrusion_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "brim_flow_ratio": 0.25,
        "bridge_flow": 0.5,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

    let brim_e = extrusion
        .extrusion_per_mm(PrintPathRole::Brim, 0.2)
        .unwrap();
    let bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::Bridge, 0.2)
        .unwrap();
    let skirt_e = extrusion
        .extrusion_per_mm(PrintPathRole::Skirt, 0.2)
        .unwrap();
    let infill_e = extrusion
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let external_e = extrusion
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (brim_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (skirt_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
    assert_eq!(
        (infill_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
    assert_eq!(
        (external_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn omitted_sparse_infill_flow_ratio_keeps_sparse_infill_extrusion_unscaled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

    let infill_e = extrusion
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (infill_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn parsed_sparse_infill_flow_ratio_reaches_extrusion_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "set_other_flow_ratios": true,
        "sparse_infill_flow_ratio": 0.25,
        "brim_flow_ratio": 0.5,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

    let infill_e = extrusion
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();
    let brim_e = extrusion
        .extrusion_per_mm(PrintPathRole::Brim, 0.2)
        .unwrap();
    let bridge_e = extrusion
        .extrusion_per_mm(PrintPathRole::Bridge, 0.2)
        .unwrap();
    let perimeter_e = extrusion
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (infill_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
    assert_eq!(
        (brim_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
    assert_eq!(
        (perimeter_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn accepts_orca_sparse_infill_flow_ratio_bounds() {
    for sparse_infill_flow_ratio in [0.0, 2.0] {
        let options: SliceOptions = serde_json::from_value(json!({
            "sparse_infill_flow_ratio": sparse_infill_flow_ratio
        }))
        .unwrap();
        assert!(options.extrusion_options().is_ok());
    }
}

#[test]
fn rejects_invalid_sparse_infill_flow_ratio_values() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "sparse_infill_flow_ratio": value })).unwrap();
        assert!(options.extrusion_options().is_err());
    }
}

#[test]
fn omitted_wall_flow_ratios_keep_perimeter_extrusion_unscaled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    for role in [
        PrintPathRole::ExternalPerimeter,
        PrintPathRole::InternalPerimeter,
    ] {
        let perimeter_e = extrusion.extrusion_per_mm(role, 0.2).unwrap();
        assert_eq!(
            (perimeter_e * 1_000_000.0).round(),
            (base_e * 1_000_000.0).round()
        );
    }
}

#[test]
fn parsed_wall_flow_ratios_reach_extrusion_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "set_other_flow_ratios": true,
        "outer_wall_flow_ratio": 0.25,
        "inner_wall_flow_ratio": 1.5,
        "brim_flow_ratio": 0.5,
        "sparse_infill_flow_ratio": 0.75,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    let external_e = extrusion
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let internal_e = extrusion
        .extrusion_per_mm(PrintPathRole::InternalPerimeter, 0.2)
        .unwrap();
    let brim_e = extrusion
        .extrusion_per_mm(PrintPathRole::Brim, 0.2)
        .unwrap();
    let sparse_infill_e = extrusion
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();
    let skirt_e = extrusion
        .extrusion_per_mm(PrintPathRole::Skirt, 0.2)
        .unwrap();

    assert_eq!(
        (external_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
    assert_eq!(
        (internal_e * 1_000_000.0).round(),
        (base_e * 1.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (brim_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (sparse_infill_e * 1_000_000.0).round(),
        (base_e * 0.75 * 1_000_000.0).round()
    );
    assert_eq!(
        (skirt_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn accepts_orca_wall_flow_ratio_bounds() {
    for key in ["outer_wall_flow_ratio", "inner_wall_flow_ratio"] {
        for value in [0.0, 2.0] {
            let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
            assert!(options.extrusion_options().is_ok());
        }
    }
}

#[test]
fn rejects_invalid_wall_flow_ratio_values() {
    for key in ["outer_wall_flow_ratio", "inner_wall_flow_ratio"] {
        for value in [
            json!(-0.1),
            json!(2.1),
            json!("not-a-number"),
            json!("NaN"),
            json!("inf"),
            json!("-inf"),
        ] {
            let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
            assert!(options.extrusion_options().is_err());
        }
    }
}

#[test]
fn accepts_orca_brim_flow_ratio_bounds() {
    for brim_flow_ratio in [0.0, 2.0] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_flow_ratio": brim_flow_ratio })).unwrap();
        assert!(options.extrusion_options().is_ok());
    }
}

#[test]
fn rejects_invalid_brim_flow_ratio_values() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_flow_ratio": value })).unwrap();
        assert!(options.extrusion_options().is_err());
    }
}

#[test]
fn parsed_bridge_speed_reaches_speed_options() {
    let options: SliceOptions = serde_json::from_value(json!({"bridge_speed": 30})).unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Bridge),
        30.0
    );
}

#[test]
fn parsed_internal_bridge_speed_reaches_speed_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_speed": 20,
        "internal_bridge_speed": "150%"
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Bridge),
        20.0
    );
    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::InternalBridge),
        30.0
    );
}
