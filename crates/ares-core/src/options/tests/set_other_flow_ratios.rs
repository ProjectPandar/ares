use super::super::*;
use crate::{ExtrusionOptions, PrintPathRole};
use serde_json::json;

fn base_e() -> f64 {
    ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0)
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap()
}

fn assert_rounded_eq(actual: f64, expected: f64) {
    assert_eq!(
        (actual * 1_000_000.0).round(),
        (expected * 1_000_000.0).round()
    );
}

fn gated_options(gate: Option<bool>) -> SliceOptions {
    let mut value = json!({
        "first_layer_flow_ratio": 0.5,
        "outer_wall_flow_ratio": 0.25,
        "inner_wall_flow_ratio": 1.5,
        "sparse_infill_flow_ratio": 0.75,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    });
    if let Some(gate) = gate {
        value["set_other_flow_ratios"] = json!(gate);
    }
    serde_json::from_value(value).unwrap()
}

#[test]
fn omitted_set_other_flow_ratios_disables_supported_other_flow_ratios() {
    assert_gate_disabled(gated_options(None));
}

#[test]
fn false_set_other_flow_ratios_disables_supported_other_flow_ratios() {
    assert_gate_disabled(gated_options(Some(false)));
}

fn assert_gate_disabled(options: SliceOptions) {
    let extrusion = options.extrusion_options().unwrap();
    let base = base_e();

    assert_rounded_eq(
        extrusion
            .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
            .unwrap(),
        base,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap(),
        base,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::InternalPerimeter, 0.2)
            .unwrap(),
        base,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
            .unwrap(),
        base,
    );
}

#[test]
fn true_set_other_flow_ratios_enables_supported_other_flow_ratios() {
    let extrusion = gated_options(Some(true)).extrusion_options().unwrap();
    let base = base_e();

    assert_rounded_eq(
        extrusion
            .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
            .unwrap(),
        base * 0.5 * 0.25,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap(),
        base * 0.25,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::InternalPerimeter, 0.2)
            .unwrap(),
        base * 1.5,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
            .unwrap(),
        base * 0.75,
    );
}

#[test]
fn false_set_other_flow_ratios_does_not_gate_print_or_brim_flow() {
    let options: SliceOptions = serde_json::from_value(json!({
        "set_other_flow_ratios": false,
        "brim_flow_ratio": 0.5,
        "print_flow_ratio": 1.5,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = base_e();

    assert_rounded_eq(
        extrusion.extrusion_per_mm(PrintPathRole::Brim, 0.2).unwrap(),
        base * 0.75,
    );
    assert_rounded_eq(
        extrusion
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap(),
        base * 1.5,
    );
}

#[test]
fn rejects_invalid_set_other_flow_ratios_values() {
    for value in [json!("true"), json!(1), json!(null), json!({})] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "set_other_flow_ratios": value })).unwrap();
        assert!(options.extrusion_options().is_err());
    }
}
