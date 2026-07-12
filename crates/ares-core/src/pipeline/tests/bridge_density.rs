use super::*;
use serde_json::json;

#[test]
fn bridge_density_changes_unsupported_external_bridge_gcode_line_count() {
    let default = bridge_gcode(options(json!({ "bridge_density": 100 })));
    let lower = bridge_gcode(options(json!({ "bridge_density": 50 })));

    assert_eq!(count_role(&default, "bridge"), 10);
    assert_eq!(count_role(&lower, "bridge"), 5);
}

#[test]
fn bridge_density_does_not_change_supported_bottom_surface_gcode_line_count() {
    let default = supported_bottom_surface_gcode(options(json!({ "bridge_density": 100 })));
    let lower = supported_bottom_surface_gcode(options(json!({ "bridge_density": 50 })));

    assert_eq!(count_role(&default, "bottom_surface"), 10);
    assert_eq!(count_role(&lower, "bottom_surface"), 10);
    assert_eq!(count_role(&lower, "bridge"), 0);
}

#[test]
fn bridge_density_composes_with_bridge_angle() {
    let gcode = bridge_gcode(options(json!({
        "bridge_density": 50,
        "bridge_angle": 90
    })));

    assert_eq!(count_role(&gcode, "bridge"), 5);
    assert!(gcode.contains(";PRINT_PATH:bridge:14,0.4 -> 10,0.4"));
}

fn bridge_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    layer_section(&gcode, 1).to_owned()
}

fn supported_bottom_surface_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 2);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    layer_section(&gcode, 1).to_owned()
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
