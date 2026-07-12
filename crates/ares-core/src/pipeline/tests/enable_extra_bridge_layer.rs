use super::*;
use serde_json::json;

#[test]
fn external_bridge_only_emits_second_external_bridge_layer() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only"
    })));

    assert_eq!(count_role(layer_section(&gcode, 1), "bridge"), 10);
    assert_eq!(count_role(layer_section(&gcode, 2), "bridge"), 10);
    assert!(!layer_section(&gcode, 2).contains(";PRINT_PATH:bottom_surface:"));
    assert!(layer_section(&gcode, 2).contains(";SPEED:print:bridge:"));
    assert!(layer_section(&gcode, 2).contains(";EXTRUSION:print:bridge:"));
}

#[test]
fn disabled_keeps_layer_above_external_bridge_as_bottom_surface() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "disabled"
    })));

    assert_eq!(count_role(layer_section(&gcode, 1), "bridge"), 10);
    assert_eq!(count_role(layer_section(&gcode, 2), "bridge"), 0);
    assert_eq!(count_role(layer_section(&gcode, 2), "bottom_surface"), 10);
}

#[test]
fn internal_only_does_not_emit_second_external_bridge_layer() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "internal_bridge_only"
    })));

    assert_eq!(count_role(layer_section(&gcode, 1), "bridge"), 10);
    assert_eq!(count_role(layer_section(&gcode, 2), "bridge"), 0);
    assert_eq!(count_role(layer_section(&gcode, 2), "bottom_surface"), 10);
}

#[test]
fn apply_to_all_emits_second_external_bridge_layer() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "apply_to_all"
    })));

    assert_eq!(count_role(layer_section(&gcode, 2), "bridge"), 10);
}

#[test]
fn extra_external_bridge_layer_composes_with_density_and_angle() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "bridge_density": 50,
        "bridge_angle": 90
    })));
    let layer = layer_section(&gcode, 2);

    assert_eq!(count_role(layer, "bridge"), 5);
    assert!(layer.contains(";PRINT_PATH:bridge:14,0.4 -> 10,0.4"));
}

#[test]
fn extra_external_bridge_layer_uses_bridge_speed_and_flow() {
    let gcode = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "bridge_speed": 17,
        "bridge_flow": 0.5
    })));
    let layer = layer_section(&gcode, 2);

    assert!(layer.contains(";SPEED:print:bridge:"));
    assert!(layer.contains(":1020"));
    assert!(layer.contains(";EXTRUSION:print:bridge:"));
    assert!(!layer.contains(";SPEED:print:bottom_surface:"));
    assert!(!layer.contains(";EXTRUSION:print:bottom_surface:"));
}

#[test]
fn extra_external_bridge_layer_extrusion_changes_with_bridge_flow() {
    let low = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "bridge_flow": 0.5
    })));
    let high = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "bridge_flow": 1.0
    })));

    assert_ne!(
        first_bridge_extrusion_line(layer_section(&low, 2)),
        first_bridge_extrusion_line(layer_section(&high, 2))
    );
}

#[test]
fn extra_external_bridge_layer_uses_thick_bridge_extrusion() {
    let thin = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "thick_bridges": false,
        "filament_diameter": [2.0]
    })));
    let thick = extra_bridge_gcode(options(json!({
        "enable_extra_bridge_layer": "external_bridge_only",
        "thick_bridges": true,
        "filament_diameter": [2.0]
    })));

    assert_ne!(
        first_bridge_extrusion_line(layer_section(&thin, 2)),
        first_bridge_extrusion_line(layer_section(&thick, 2))
    );
}

fn extra_bridge_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            rectangle_contour(0.0, 0.0, 4.0, 4.0),
            rectangle_contour(10.0, 0.0, 14.0, 4.0),
            rectangle_contour(10.0, 0.0, 14.0, 4.0),
        ],
    );
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn rectangle_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<crate::Contour> {
    vec![crate::Contour::new(vec![
        crate::Point2::new(min_x, min_y),
        crate::Point2::new(max_x, min_y),
        crate::Point2::new(max_x, max_y),
        crate::Point2::new(min_x, max_y),
    ])]
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 3,
        "top_shell_layers": 0,
        "bridge_no_support": true,
        "bridge_speed": 20,
        "bottom_surface_speed": 60,
        "bottom_surface_pattern": "alignedrectilinear",
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn layer_section(gcode: &str, layer_index: usize) -> &str {
    let marker = format!(";LAYER:{layer_index}");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    let next = format!(";LAYER:{}", layer_index + 1);
    rest.find(&next).map_or(rest, |end| &rest[..end])
}

fn count_role(layer: &str, role: &str) -> usize {
    layer
        .lines()
        .filter(|line| line.starts_with(&format!(";PRINT_PATH:{role}:")))
        .count()
}

fn first_bridge_extrusion_line(layer: &str) -> &str {
    layer
        .lines()
        .find(|line| line.starts_with(";EXTRUSION:print:bridge:"))
        .unwrap()
}
