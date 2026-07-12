use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn initial_layer_line_width_changes_first_layer_perimeter_gcode_extrusion_delta() {
    let base: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let wide: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "initial_layer_line_width": 0.6,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let base_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&base), &base).unwrap()).unwrap();
    let wide_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&wide), &wide).unwrap()).unwrap();

    assert!(
        first_extrusion_delta(&wide_gcode, "external_perimeter")
            > first_extrusion_delta(&base_gcode, "external_perimeter")
    );
}

#[test]
fn initial_layer_line_width_does_not_change_second_layer_perimeter_gcode_extrusion_delta() {
    let base: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();
    let wide: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "initial_layer_line_width": 0.6,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let base_pipeline =
        super::run_slicing_pipeline(super::square_pyramid_ascii_stl(), &base).unwrap();
    let wide_pipeline =
        super::run_slicing_pipeline(super::square_pyramid_ascii_stl(), &wide).unwrap();
    let base_gcode = String::from_utf8(format_gcode(&base_pipeline, &base).unwrap()).unwrap();
    let wide_gcode = String::from_utf8(format_gcode(&wide_pipeline, &wide).unwrap()).unwrap();

    assert!(
        (layer_extrusion_delta(&wide_gcode, 1, "external_perimeter")
            - layer_extrusion_delta(&base_gcode, 1, "external_perimeter"))
        .abs()
            <= 0.000002
    );
}

fn first_extrusion_delta(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let target = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
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
    panic!("missing {role} extrusion");
}

fn layer_extrusion_delta(gcode: &str, layer_id: usize, role: &str) -> f64 {
    let mut current_layer = None;
    let mut previous_e = 0.0;
    let target = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if current_layer == Some(layer_id) && line.starts_with(&target) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing layer {layer_id} {role} extrusion");
}
