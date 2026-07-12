use super::super::*;
use crate::{PrintPathRole, ShellLayerOptions, ToolpathMoveKind};
use serde_json::json;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn parses_top_surface_width_and_flow_options_without_other_flow_gate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0,
        "top_surface_line_width": 0,
        "filament_diameter": [2.0],
        "top_solid_infill_flow_ratio": 0.5,
        "bottom_solid_infill_flow_ratio": 0.25
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let default: SliceOptions = serde_json::from_value(json!({
        "line_width": 0,
        "top_surface_line_width": 0,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let base = default.extrusion_options().unwrap();

    assert_eq!(extrusion.width_for_role(PrintPathRole::TopSolidInfill), 0.4);
    assert_eq!(
        round_6(
            extrusion
                .extrusion_per_mm(PrintPathRole::TopSolidInfill, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::TopSolidInfill, 0.2)
                .unwrap()
                * 0.5
        )
    );
    assert_eq!(
        round_6(
            extrusion
                .extrusion_per_mm(PrintPathRole::BottomSurface, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::BottomSurface, 0.2)
                .unwrap()
                * 0.25
        )
    );
}

#[test]
fn rejects_invalid_top_bottom_surface_numeric_options() {
    for (key, value) in [
        ("top_solid_infill_flow_ratio", json!(-0.1)),
        ("top_solid_infill_flow_ratio", json!(2.1)),
        ("bottom_solid_infill_flow_ratio", json!(-0.1)),
        ("bottom_solid_infill_flow_ratio", json!(2.1)),
        ("top_surface_line_width", json!(-1)),
        ("top_surface_speed", json!(0)),
        ("top_surface_acceleration", json!(-1)),
        ("top_surface_jerk", json!(-1)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = if key.contains("flow") || key.contains("line_width") {
            options.extrusion_options().unwrap_err()
        } else {
            options.speed_options().unwrap_err()
        };

        assert!(err.to_string().contains(key), "{err}");
    }
}

#[test]
fn parses_top_surface_speed_acceleration_and_jerk() {
    let options: SliceOptions = serde_json::from_value(json!({
        "top_surface_speed": 123,
        "default_acceleration": 700,
        "top_surface_acceleration": "25%",
        "default_jerk": 8,
        "top_surface_jerk": 3
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill),
        123.0
    );
    assert_eq!(
        speeds.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::TopSolidInfill,
            false
        ),
        Some(175.0)
    );
    assert_eq!(
        speeds.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill, false),
        Some(3.0)
    );
}

#[test]
fn shell_layer_options_parse_orca_defaults() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();
    let shell_layers = options.shell_layer_options().unwrap();

    assert_eq!(shell_layers.bottom_shell_layers(), 3);
    assert_eq!(shell_layers.top_shell_layers(), 4);
}

#[test]
fn shell_layer_options_parse_orca_thickness_defaults() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();
    let shell_layers = options.shell_layer_options().unwrap();

    assert_eq!(shell_layers.bottom_shell_thickness_mm(), 0.0);
    assert_eq!(shell_layers.top_shell_thickness_mm(), 0.6);
    assert_eq!(ShellLayerOptions::new(3, 4), ShellLayerOptions::default());
    fn assert_eq_trait<T: Eq>() {}
    assert_eq_trait::<ShellLayerOptions>();
}

#[test]
fn shell_layer_options_parse_explicit_thicknesses() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bottom_shell_thickness": "0.45",
        "top_shell_thickness": 0.8
    }))
    .unwrap();
    let shell_layers = options.shell_layer_options().unwrap();

    assert_eq!(shell_layers.bottom_shell_thickness_mm(), 0.45);
    assert_eq!(shell_layers.top_shell_thickness_mm(), 0.8);
}

#[test]
fn shell_layer_options_reject_invalid_thicknesses() {
    for (key, value) in [
        ("bottom_shell_thickness", json!(-0.1)),
        ("bottom_shell_thickness", json!("abc")),
        ("bottom_shell_thickness", json!("NaN")),
        ("bottom_shell_thickness", json!("inf")),
        ("top_shell_thickness", json!(-0.1)),
        ("top_shell_thickness", json!("abc")),
        ("top_shell_thickness", json!("NaN")),
        ("top_shell_thickness", json!("-inf")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = options.shell_layer_options().unwrap_err();

        assert!(err.to_string().contains(key), "{err}");
    }
}

#[test]
fn shell_layer_options_parse_explicit_counts() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bottom_shell_layers": "2",
        "top_shell_layers": 1
    }))
    .unwrap();
    let shell_layers = options.shell_layer_options().unwrap();

    assert_eq!(shell_layers.bottom_shell_layers(), 2);
    assert_eq!(shell_layers.top_shell_layers(), 1);
}

#[test]
fn shell_layer_options_reject_invalid_counts() {
    for (key, value) in [
        ("bottom_shell_layers", json!(-1)),
        ("top_shell_layers", json!(1.5)),
        ("top_shell_layers", json!("abc")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = options.shell_layer_options().unwrap_err();

        assert!(err.to_string().contains(key), "{err}");
    }
}
