use super::*;
use serde_json::json;

#[test]
fn positive_internal_bridge_angle_changes_internal_bridge_gcode_direction() {
    let gcode = internal_bridge_angle_gcode(options(json!({
        "internal_bridge_density": 50,
        "internal_bridge_angle": 90
    })));
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains(";PRINT_PATH:internal_bridge:4,0.4 -> 0,0.4"));
    assert!(!layer.contains(";PRINT_PATH:internal_bridge:0.4,0 -> 0.4,4"));
}

#[test]
fn zero_internal_bridge_angle_auto_detects_non_square_internal_bridge_gcode_direction() {
    let options = options(json!({
        "internal_bridge_density": 50,
        "internal_bridge_filter": "nofilter",
        "internal_bridge_angle": 0
    }));
    let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            rectangle_contour(0.0, 0.0, 4.0, 2.0),
            rectangle_contour(0.0, 0.0, 4.0, 2.0),
            rectangle_contour(0.0, 0.0, 4.0, 2.0),
        ],
    );
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains(";PRINT_PATH:internal_bridge:4,0.4 -> 0,0.4"));
    assert!(!layer.contains(";PRINT_PATH:internal_bridge:0.4,0 -> 0.4,2"));
}

#[test]
fn internal_bridge_angle_does_not_create_internal_bridge_gcode_at_default_density() {
    let gcode = internal_bridge_angle_gcode(options(json!({
        "internal_bridge_density": 100,
        "internal_bridge_angle": 90
    })));
    let layer = layer_section(&gcode, 1);

    assert_eq!(count_role(layer, "internal_bridge"), 0);
    assert_eq!(count_role(layer, "solid_infill"), 10);
    assert!(layer.contains(";PRINT_PATH:solid_infill:0.2,0 -> 0.2,4"));
    assert!(!layer.contains(";PRINT_PATH:solid_infill:4,0.2 -> 0,0.2"));
}

fn internal_bridge_angle_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "solid_infill_direction": 0,
        "solid_infill_rotate_template": "0",
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn rectangle_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<crate::Contour> {
    vec![crate::Contour::new(vec![
        crate::Point2::new(min_x, min_y),
        crate::Point2::new(max_x, min_y),
        crate::Point2::new(max_x, max_y),
        crate::Point2::new(min_x, max_y),
    ])]
}

fn layer_section(gcode: &str, layer_index: usize) -> &str {
    let marker = format!(";LAYER:{layer_index}");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    let next = format!(";LAYER:{}", layer_index + 1);
    rest.find(&next).map_or(rest, |end| &rest[..end])
}

fn count_role(gcode: &str, role: &str) -> usize {
    gcode
        .lines()
        .filter(|line| line.starts_with(&format!(";PRINT_PATH:{role}:")))
        .count()
}
