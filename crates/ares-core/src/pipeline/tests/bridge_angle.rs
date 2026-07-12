use super::*;
use serde_json::json;

#[test]
fn bridge_angle_changes_unsupported_external_bridge_gcode_direction() {
    let options = options(json!({
        "bridge_no_support": true,
        "bridge_angle": 90,
        "bottom_surface_pattern": "alignedrectilinear"
    }));

    let pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains(";PRINT_PATH:bridge:14,0.2 -> 10,0.2"));
    assert!(!layer.contains(";PRINT_PATH:bridge:10.2,0 -> 10.2,4"));
}

#[test]
fn bridge_angle_does_not_change_supported_bottom_surface_gcode_direction() {
    let options = options(json!({
        "bridge_no_support": true,
        "bridge_angle": 90,
        "bottom_surface_pattern": "alignedrectilinear"
    }));

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 2);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains(";PRINT_PATH:bottom_surface:0.2,0 -> 0.2,4"));
    assert!(!layer.contains(";PRINT_PATH:bottom_surface:4,0.2 -> 0,0.2"));
}

#[test]
fn zero_bridge_angle_auto_detects_unsupported_external_bridge_gcode_direction() {
    let options = options(json!({
        "bridge_no_support": true,
        "bridge_angle": 0,
        "bottom_surface_pattern": "alignedrectilinear"
    }));

    let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            rectangle_contour(0.0, 0.0, 4.0, 2.0),
            rectangle_contour(10.0, 0.0, 14.0, 2.0),
        ],
    );
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains(";PRINT_PATH:bridge:14,0.2 -> 10,0.2"));
    assert!(!layer.contains(";PRINT_PATH:bridge:10.2,0 -> 10.2,2"));
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
