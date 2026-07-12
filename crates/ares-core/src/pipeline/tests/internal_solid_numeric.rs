use super::*;
use serde_json::json;

#[test]
fn internal_solid_numeric_options_change_solid_gcode_independently() {
    let low = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "set_other_flow_ratios": true,
        "sparse_infill_line_width": 0.3,
        "internal_solid_infill_line_width": 0.4,
        "sparse_infill_flow_ratio": 1.5,
        "internal_solid_infill_flow_ratio": 0.5,
        "sparse_infill_speed": 70,
        "internal_solid_infill_speed": 40,
        "initial_layer_infill_speed": 10,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let high = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "set_other_flow_ratios": true,
        "sparse_infill_line_width": 0.3,
        "internal_solid_infill_line_width": 0.8,
        "sparse_infill_flow_ratio": 0.5,
        "internal_solid_infill_flow_ratio": 1.5,
        "sparse_infill_speed": 70,
        "internal_solid_infill_speed": 90,
        "initial_layer_infill_speed": 10,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));

    let low_gcode = gcode(
        &crate::pipeline::test_support::rectangular_layers_pipeline(&low, 3),
        &low,
    );
    let high_gcode = gcode(
        &crate::pipeline::test_support::rectangular_layers_pipeline(&high, 3),
        &high,
    );

    assert!(
        first_extrusion_delta_after_layer(&high_gcode, 1, "solid_infill")
            > first_extrusion_delta_after_layer(&low_gcode, 1, "solid_infill")
    );
    assert_eq!(
        first_layer_speed_feedrate(&low_gcode, 1, "solid_infill"),
        2400.0
    );
    assert_eq!(
        first_layer_speed_feedrate(&high_gcode, 1, "solid_infill"),
        5400.0
    );
    assert!(low_gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(low_gcode.contains(";PRINT_PATH:top_solid_infill:"));
}

#[test]
fn internal_solid_acceleration_changes_non_first_layer_solid_gcode() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "default_acceleration": 800,
        "initial_layer_acceleration": 0,
        "sparse_infill_acceleration": "75%",
        "internal_solid_infill_acceleration": "25%"
    }));

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let output = gcode(&pipeline, &options);

    assert_acceleration_after_layer_move_marker(
        &output,
        1,
        "M204 S200",
        ";MOVE:print:solid_infill:",
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn gcode(pipeline: &SlicingPipeline, options: &SliceOptions) -> String {
    String::from_utf8(crate::gcode::format_gcode(pipeline, options).unwrap()).unwrap()
}

fn first_extrusion_delta_after_layer(gcode: &str, layer_id: usize, role: &str) -> f64 {
    let layer_marker = format!(";LAYER:{layer_id}");
    let layer_start = gcode
        .lines()
        .position(|line| line == layer_marker.as_str())
        .unwrap_or_else(|| panic!("missing {layer_marker}"));
    let target = format!(";EXTRUSION:print:{role}:");
    let mut previous_e = 0.0;
    for line in gcode.lines().skip(layer_start) {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&target) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing layer {layer_id} {role} extrusion");
}

fn first_layer_speed_feedrate(gcode: &str, layer_id: usize, role: &str) -> f64 {
    let layer_marker = format!(";LAYER:{layer_id}");
    let layer_start = gcode
        .lines()
        .position(|line| line == layer_marker.as_str())
        .unwrap_or_else(|| panic!("missing {layer_marker}"));
    let target = format!(";SPEED:print:{role}:");
    gcode
        .lines()
        .skip(layer_start)
        .find_map(|line| line.strip_prefix(&target))
        .and_then(|rest| rest.rsplit(':').next())
        .and_then(|feedrate| feedrate.parse().ok())
        .unwrap_or_else(|| panic!("missing layer {layer_id} {role} speed"))
}

fn assert_acceleration_after_layer_move_marker(
    output: &str,
    layer_id: usize,
    acceleration: &str,
    move_prefix: &str,
) {
    let layer_marker = format!(";LAYER:{layer_id}");
    let layer_start = output
        .lines()
        .position(|line| line == layer_marker.as_str())
        .unwrap_or_else(|| panic!("missing {layer_marker}"));
    let lines = output.lines().skip(layer_start).collect::<Vec<_>>();
    let marker_index = lines
        .iter()
        .position(|line| line.starts_with(move_prefix))
        .unwrap_or_else(|| panic!("missing {move_prefix}"));

    assert_eq!(
        lines[marker_index + 1..]
            .iter()
            .copied()
            .find(|line| !line.starts_with(';')),
        Some(acceleration)
    );
}
