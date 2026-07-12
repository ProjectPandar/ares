use super::*;
use serde_json::json;

#[test]
fn top_surface_density_changes_only_top_surface_gcode_line_count() {
    let default = surface_gcode(options(json!({ "top_surface_density": 100 })));
    let lower = surface_gcode(options(json!({ "top_surface_density": 50 })));

    assert_eq!(count_role(&default, "bottom_surface"), 10);
    assert_eq!(count_role(&default, "top_solid_infill"), 10);
    assert_eq!(count_role(&lower, "bottom_surface"), 10);
    assert_eq!(count_role(&lower, "top_solid_infill"), 5);
}

#[test]
fn bottom_surface_density_changes_only_bottom_surface_gcode_line_count() {
    let default = surface_gcode(options(json!({ "bottom_surface_density": 100 })));
    let lower = surface_gcode(options(json!({ "bottom_surface_density": 50 })));

    assert_eq!(count_role(&default, "bottom_surface"), 10);
    assert_eq!(count_role(&default, "top_solid_infill"), 10);
    assert_eq!(count_role(&lower, "bottom_surface"), 5);
    assert_eq!(count_role(&lower, "top_solid_infill"), 10);
}

#[test]
fn elephant_foot_layers_density_changes_only_internal_solid_gcode_count() {
    let default = surface_gcode(options(json!({
        "elefant_foot_layers_density": 100,
        "elefant_foot_compensation_layers": 2
    })));
    let lower = surface_gcode(options(json!({
        "elefant_foot_layers_density": 50,
        "elefant_foot_compensation_layers": 2
    })));

    assert_eq!(count_role(&default, "bottom_surface"), 10);
    assert_eq!(count_role(&default, "solid_infill"), 10);
    assert_eq!(count_role(&default, "top_solid_infill"), 10);
    assert_eq!(count_role(&lower, "bottom_surface"), 10);
    assert_eq!(count_role(&lower, "solid_infill"), 5);
    assert_eq!(count_role(&lower, "top_solid_infill"), 10);
}

#[test]
fn zero_top_surface_density_omits_top_surface_gcode_paths() {
    let gcode = surface_gcode(options(json!({ "top_surface_density": 0 })));

    assert_eq!(count_role(&gcode, "bottom_surface"), 10);
    assert_eq!(count_role(&gcode, "top_solid_infill"), 0);
}

#[test]
fn min_width_top_surface_suppresses_rectangular_top_surface_gcode() {
    let gcode = surface_gcode(options(json!({
        "min_width_top_surface": 5.0
    })));

    assert_eq!(count_role(&gcode, "bottom_surface"), 10);
    assert_eq!(count_role(&gcode, "top_solid_infill"), 0);
}

#[test]
fn min_width_top_surface_zero_preserves_rectangular_top_surface_gcode() {
    let gcode = surface_gcode(options(json!({
        "min_width_top_surface": 0
    })));

    assert_eq!(count_role(&gcode, "bottom_surface"), 10);
    assert_eq!(count_role(&gcode, "top_solid_infill"), 10);
}

fn surface_gcode(options: SliceOptions) -> String {
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
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "bottom_surface_pattern": "alignedrectilinear",
        "top_surface_pattern": "alignedrectilinear",
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn count_role(gcode: &str, role: &str) -> usize {
    gcode
        .lines()
        .filter(|line| line.starts_with(&format!(";PRINT_PATH:{role}:")))
        .count()
}
