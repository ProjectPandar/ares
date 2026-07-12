use super::*;
use serde_json::json;

#[test]
fn small_area_infill_flow_compensation_reduces_solid_surface_gcode_e_delta() {
    let disabled = options(json!({ "small_area_infill_flow_compensation": false }));
    let enabled = options(json!({ "small_area_infill_flow_compensation": true }));

    let disabled_gcode = gcode(&disabled);
    let enabled_gcode = gcode(&enabled);

    assert!(
        first_role_extrusion_delta(&enabled_gcode, "bottom_surface")
            < first_role_extrusion_delta(&disabled_gcode, "bottom_surface")
    );
}

fn gcode(options: &SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(options, 3);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap()
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
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "bottom_surface_pattern": "alignedrectilinear",
        "internal_solid_infill_pattern": "alignedrectilinear",
        "top_surface_pattern": "alignedrectilinear",
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    value.as_object_mut().unwrap().extend(
        extra
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}

fn first_role_extrusion_delta(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let prefix = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&prefix) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing {role} extrusion");
}
