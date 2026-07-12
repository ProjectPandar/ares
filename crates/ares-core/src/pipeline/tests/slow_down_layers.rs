use crate::{
    PrintPathRole, SliceOptions, gcode::format_gcode, pipeline::test_support::single_path_pipeline,
};
use serde_json::json;

#[test]
fn slow_down_layers_interpolates_layer_one_perimeter_gcode_feedrate() {
    let options = options(json!({
        "slow_down_layers": 4,
        "outer_wall_speed": 90,
        "initial_layer_speed": 30
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        2700.0
    );
}

#[test]
fn slow_down_layers_stop_at_configured_layer_boundary() {
    let options = options(json!({
        "slow_down_layers": 4,
        "outer_wall_speed": 90,
        "initial_layer_speed": 30
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 4);

    assert_eq!(
        layer_speed_feedrate(&gcode, 4, "external_perimeter", "print"),
        5400.0
    );
}

#[test]
fn dont_slow_down_outer_wall_preserves_external_perimeter_gcode_feedrate() {
    let options = options(json!({
        "slow_down_layers": 4,
        "dont_slow_down_outer_wall": true,
        "outer_wall_speed": 90,
        "initial_layer_speed": 30
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "external_perimeter", "print"),
        5400.0
    );
}

#[test]
fn dont_slow_down_outer_wall_leaves_internal_perimeter_interpolation_enabled() {
    let options = options(json!({
        "slow_down_layers": 4,
        "dont_slow_down_outer_wall": true,
        "outer_wall_speed": 90,
        "inner_wall_speed": 90,
        "initial_layer_speed": 30
    }));

    let gcode = layer_gcode(&options, PrintPathRole::InternalPerimeter, 1);

    assert_eq!(
        layer_speed_feedrate(&gcode, 1, "internal_perimeter", "print"),
        2700.0
    );
}

#[test]
fn slow_down_layers_do_not_interpolate_skirt_gcode_feedrate() {
    let options = options(json!({
        "slow_down_layers": 4,
        "skirt_speed": 55,
        "outer_wall_speed": 90,
        "initial_layer_speed": 30
    }));

    let gcode = layer_gcode(&options, PrintPathRole::Skirt, 1);

    assert_eq!(layer_speed_feedrate(&gcode, 1, "skirt", "print"), 3300.0);
}

#[test]
fn layer_time_slowdown_changes_print_gcode_feedrate() {
    let options = options(json!({
        "slow_down_for_layer_cooling": true,
        "slow_down_layer_time": 0.2,
        "slow_down_min_speed": 1,
        "outer_wall_speed": 100,
        "initial_layer_speed": 100
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 0);

    assert!(
        (layer_speed_feedrate(&gcode, 0, "external_perimeter", "print") - 599.4005994005994).abs()
            <= 0.000001
    );
}

#[test]
fn fan_cooling_layer_time_interpolates_part_cooling_fan_gcode() {
    let options = options(json!({
        "slow_down_for_layer_cooling": false,
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "slow_down_layer_time": 0.0,
        "fan_cooling_layer_time": 2.0,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "outer_wall_speed": 2.0,
        "initial_layer_speed": 2.0
    }));

    let gcode = layer_gcode(&options, PrintPathRole::ExternalPerimeter, 0);

    assert_eq!(fan_lines(&gcode), vec!["M106 S153"]);
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn layer_gcode(options: &SliceOptions, role: PrintPathRole, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, role, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
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

fn fan_lines(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with("M106 "))
        .collect()
}
