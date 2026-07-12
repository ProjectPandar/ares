use super::*;
use serde_json::json;

#[test]
fn bridge_no_support_true_emits_unsupported_bottom_dense_infill_as_bridge() {
    let off = options(json!({ "bridge_no_support": false }));
    let on = options(json!({ "bridge_no_support": true }));

    let off_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&off);
    let on_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&on);
    let off_gcode =
        String::from_utf8(crate::gcode::format_gcode(&off_pipeline, &off).unwrap()).unwrap();
    let on_gcode =
        String::from_utf8(crate::gcode::format_gcode(&on_pipeline, &on).unwrap()).unwrap();

    assert!(layer_section(&off_gcode, 1).contains(";PRINT_PATH:bottom_surface:"));
    assert!(!layer_section(&off_gcode, 1).contains(";PRINT_PATH:bridge:"));
    assert!(layer_section(&on_gcode, 1).contains(";PRINT_PATH:bridge:"));
    assert!(!layer_section(&on_gcode, 1).contains(";PRINT_PATH:bottom_surface:"));
    assert_ne!(off_gcode, on_gcode);
}

#[test]
fn counterbore_sacrificial_layer_keeps_unsupported_bottom_surface_gcode_role() {
    let default = options(json!({ "bridge_no_support": true }));
    let partial = options(json!({
        "bridge_no_support": true,
        "counterbore_hole_bridging": "partiallybridge"
    }));
    let sacrificial = options(json!({
        "bridge_no_support": true,
        "counterbore_hole_bridging": "sacrificiallayer"
    }));

    let default_gcode = gcode_for_unsupported_second_layer(&default);
    let partial_gcode = gcode_for_unsupported_second_layer(&partial);
    let sacrificial_gcode = gcode_for_unsupported_second_layer(&sacrificial);
    let default_layer = layer_section(&default_gcode, 1);
    let partial_layer = layer_section(&partial_gcode, 1);
    let sacrificial_layer = layer_section(&sacrificial_gcode, 1);

    assert!(default_layer.contains(";PRINT_PATH:bridge:"));
    assert!(default_layer.contains("F1200"));
    assert!(partial_layer.contains(";PRINT_PATH:bridge:"));
    assert!(partial_layer.contains("F1200"));
    assert!(sacrificial_layer.contains(";PRINT_PATH:bottom_surface:"));
    assert!(sacrificial_layer.contains("F3600"));
    assert!(!sacrificial_layer.contains(";PRINT_PATH:bridge:"));
}

#[test]
fn counterbore_sacrificial_layer_does_not_apply_bridge_density_or_angle() {
    let default = options(json!({
        "bridge_no_support": true,
        "bridge_density": 50,
        "bridge_angle": 90,
        "bottom_surface_pattern": "alignedrectilinear"
    }));
    let partial = options(json!({
        "bridge_no_support": true,
        "counterbore_hole_bridging": "partiallybridge",
        "bridge_density": 50,
        "bridge_angle": 90,
        "bottom_surface_pattern": "alignedrectilinear"
    }));
    let sacrificial = options(json!({
        "bridge_no_support": true,
        "counterbore_hole_bridging": "sacrificiallayer",
        "bridge_density": 50,
        "bridge_angle": 90,
        "bottom_surface_pattern": "alignedrectilinear"
    }));

    let default_gcode = gcode_for_unsupported_second_layer(&default);
    let partial_gcode = gcode_for_unsupported_second_layer(&partial);
    let sacrificial_gcode = gcode_for_unsupported_second_layer(&sacrificial);
    let default_layer = layer_section(&default_gcode, 1);
    let partial_layer = layer_section(&partial_gcode, 1);
    let sacrificial_layer = layer_section(&sacrificial_gcode, 1);

    assert_eq!(count_role(default_layer, "bridge"), 5);
    assert!(default_layer.contains(";PRINT_PATH:bridge:14,0.4 -> 10,0.4"));
    assert_eq!(count_role(partial_layer, "bridge"), 5);
    assert!(partial_layer.contains(";PRINT_PATH:bridge:14,0.4 -> 10,0.4"));
    assert_eq!(count_role(sacrificial_layer, "bottom_surface"), 10);
    assert!(sacrificial_layer.contains(";PRINT_PATH:bottom_surface:10.2,0 -> 10.2,4"));
    assert!(!sacrificial_layer.contains(";PRINT_PATH:bottom_surface:14,0.4 -> 10,0.4"));
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
        "bottom_shell_layers": 2,
        "top_shell_layers": 0,
        "bridge_speed": 20,
        "bottom_surface_speed": 60,
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn gcode_for_unsupported_second_layer(options: &SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(options);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
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
