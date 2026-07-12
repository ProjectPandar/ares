use super::*;
use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn initial_layer_speed_changes_first_layer_perimeter_feedrate_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_speed": 70,
        "initial_layer_speed": 20,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "external_perimeter", "print"),
        1200.0
    );
    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        4200.0
    );
}

#[test]
fn initial_layer_infill_speed_changes_first_layer_sparse_infill_feedrate_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_speed": 70,
        "sparse_infill_speed": 100,
        "initial_layer_speed": 20,
        "initial_layer_infill_speed": 40,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let gcode = String::from_utf8(format_gcode(&rectangular_pipeline(&options), &options).unwrap())
        .unwrap();

    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "external_perimeter", "print"),
        1200.0
    );
    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "sparse_infill", "print"),
        2400.0
    );
}

#[test]
fn initial_layer_travel_speed_changes_first_layer_travel_feedrates_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "travel_speed": 120,
        "initial_layer_travel_speed": "50%",
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(layer_z_travel_feedrate(&gcode, 0), 3600.0);
    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "external_perimeter", "travel"),
        3600.0
    );
    assert_eq!(layer_z_travel_feedrate(&gcode, 1), 7200.0);
    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "travel"),
        7200.0
    );
}

#[test]
fn numeric_initial_layer_travel_speed_changes_first_layer_travel_feedrates() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "travel_speed": 120,
        "initial_layer_travel_speed": 45,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(layer_z_travel_feedrate(&gcode, 0), 2700.0);
    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "external_perimeter", "travel"),
        2700.0
    );
}

#[test]
fn travel_speed_z_changes_layer_z_travel_feedrates_only() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "travel_speed": 120,
        "initial_layer_travel_speed": 45,
        "travel_speed_z": 25,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(layer_z_travel_feedrate(&gcode, 0), 1500.0);
    assert_eq!(layer_z_travel_feedrate(&gcode, 1), 1500.0);
    assert_eq!(
        layer_speed_feedrate(&gcode, 0, "external_perimeter", "travel"),
        2700.0
    );
    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "travel"),
        7200.0
    );
}

#[test]
fn zero_travel_speed_z_preserves_layer_z_fallback_feedrates() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "travel_speed": 120,
        "initial_layer_travel_speed": 45,
        "travel_speed_z": 0,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(layer_z_travel_feedrate(&gcode, 0), 2700.0);
    assert_eq!(layer_z_travel_feedrate(&gcode, 1), 7200.0);
}

fn layer_speed_feedrate(gcode: &str, layer_id: usize, role: &str, kind: &str) -> f64 {
    let mut current_layer = None;
    let target = format!(";SPEED:{kind}:{role}:");
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if current_layer == Some(layer_id) && line.starts_with(&target) {
            return line.rsplit(':').next().unwrap().parse().unwrap();
        }
    }
    panic!("missing layer {layer_id} {kind} {role} speed");
}

fn layer_z_travel_feedrate(gcode: &str, layer_id: usize) -> f64 {
    let mut lines = gcode.lines();
    while let Some(line) = lines.next() {
        if line == format!(";LAYER:{layer_id}") {
            return lines
                .find_map(|line| {
                    line.strip_prefix("G1 Z")
                        .and_then(|line| line.rsplit_once(" F"))
                        .and_then(|(_, feedrate)| feedrate.parse().ok())
                })
                .unwrap();
        }
    }
    panic!("missing layer {layer_id} Z travel");
}
