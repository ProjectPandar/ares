use super::super::*;
use crate::{PrintPathRole, ToolpathMoveKind};
use serde_json::json;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn gap_fill_width_uses_line_width_fallback() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();

    assert_eq!(
        options
            .extrusion_options()
            .unwrap()
            .width_for_role(PrintPathRole::GapFill),
        0.4
    );
}

#[test]
fn gap_fill_flow_ratio_is_gated_and_keeps_first_layer_flow() {
    let gated_off: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "gap_fill_flow_ratio": 0.25,
        "first_layer_flow_ratio": 0.5
    }))
    .unwrap();
    let gated_on: SliceOptions = serde_json::from_value(json!({
        "set_other_flow_ratios": true,
        "line_width": 0.4,
        "filament_diameter": [2.0],
        "gap_fill_flow_ratio": 0.25,
        "first_layer_flow_ratio": 0.5
    }))
    .unwrap();

    let off = gated_off.extrusion_options().unwrap();
    let on = gated_on.extrusion_options().unwrap();
    let base = off.extrusion_per_mm(PrintPathRole::GapFill, 0.2).unwrap();

    assert_eq!(
        round_6(on.extrusion_per_mm(PrintPathRole::GapFill, 0.2).unwrap()),
        round_6(base * 0.25)
    );
    assert_eq!(
        round_6(
            on.extrusion_per_mm_for_layer(PrintPathRole::GapFill, 0.2, true)
                .unwrap()
        ),
        round_6(base * 0.25 * 0.5)
    );
}

#[test]
fn validates_gap_fill_flow_ratio_even_when_gate_is_off() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "gap_fill_flow_ratio": value })).unwrap();

        let err = options.extrusion_options().unwrap_err();

        assert!(err.to_string().contains("gap_fill_flow_ratio"));
    }
}

#[test]
fn gap_infill_speed_defaults_and_overrides_gap_fill_only() {
    let defaults = SliceOptions::default().speed_options().unwrap();
    assert_eq!(
        defaults.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::GapFill),
        30.0
    );

    let options: SliceOptions = serde_json::from_value(json!({
        "gap_infill_speed": 42,
        "initial_layer_speed": 12,
        "initial_layer_infill_speed": 18,
        "sparse_infill_speed": 80
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::GapFill),
        42.0
    );
    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::GapFill, true),
        42.0
    );
    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::SparseInfill),
        80.0
    );
}

#[test]
fn gap_infill_speed_accepts_zero_for_generation_gate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "gap_infill_speed": 0
    }))
    .unwrap();

    assert_eq!(
        options
            .speed_options()
            .unwrap()
            .speed_for_role(ToolpathMoveKind::Print, PrintPathRole::GapFill),
        0.0
    );
}

#[test]
fn rejects_invalid_gap_infill_speed_values() {
    for value in [json!(-1), json!("NaN"), json!("fast"), json!(false)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "gap_infill_speed": value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(err.to_string().contains("gap_infill_speed"));
    }
}

#[test]
fn gap_fill_target_defaults_to_nowhere() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    assert_eq!(
        options.gap_fill_target().unwrap().as_str_for_tests(),
        "nowhere"
    );
}

#[test]
fn parses_gap_fill_target_enum_values() {
    for value in ["everywhere", "topbottom", "nowhere"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "gap_fill_target": value
        }))
        .unwrap();

        assert_eq!(options.gap_fill_target().unwrap().as_str_for_tests(), value);
    }
}

#[test]
fn rejects_invalid_gap_fill_target_values() {
    for value in [json!("bad"), json!(true), json!(1), json!(["nowhere"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "gap_fill_target": value
        }))
        .unwrap();

        let err = options.gap_fill_target().unwrap_err();
        assert!(err.to_string().contains("gap_fill_target"));
    }
}
