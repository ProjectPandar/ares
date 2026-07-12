use super::*;
use serde_json::json;

#[test]
fn default_dense_middle_layer_still_emits_solid_infill_gcode_paths() {
    let gcode = internal_bridge_density_gcode(options(json!({
        "internal_bridge_density": 100
    })));

    assert_eq!(count_role(&gcode, "bottom_surface"), 10);
    assert_eq!(count_role(&gcode, "solid_infill"), 10);
    assert_eq!(count_role(&gcode, "internal_bridge"), 0);
    assert_eq!(count_role(&gcode, "top_solid_infill"), 10);
    assert!(gcode.contains(";SPEED:print:solid_infill:"));
    assert!(gcode.contains(";EXTRUSION:print:solid_infill:"));
}

#[test]
fn lower_internal_bridge_density_reduces_internal_bridge_gcode_line_count() {
    let default = internal_bridge_density_gcode(options(json!({
        "internal_bridge_density": 100
    })));
    let lower = internal_bridge_density_gcode(options(json!({
        "internal_bridge_density": 50
    })));

    assert_eq!(count_role(&default, "solid_infill"), 10);
    assert_eq!(count_role(&default, "internal_bridge"), 0);
    assert_eq!(count_role(&lower, "internal_bridge"), 5);
    assert_eq!(count_role(&lower, "solid_infill"), 0);
    assert_eq!(count_role(&lower, "bottom_surface"), 10);
    assert_eq!(count_role(&lower, "top_solid_infill"), 10);
    assert!(lower.contains(";SPEED:print:internal_bridge:"));
    assert!(lower.contains(";EXTRUSION:print:internal_bridge:"));
}

#[test]
fn default_filter_keeps_small_internal_bridge_candidate_as_solid_gcode() {
    let gcode = small_internal_bridge_filter_gcode(options(json!({
        "internal_bridge_density": 50
    })));

    assert_eq!(count_role(&gcode, "internal_bridge"), 0);
    assert!(gcode.contains(";PRINT_PATH:solid_infill:"));
    assert!(gcode.contains(";EXTRUSION:print:solid_infill:"));
}

#[test]
fn limited_filter_emits_internal_bridge_gcode_for_small_candidate() {
    let gcode = small_internal_bridge_filter_gcode(options(json!({
        "internal_bridge_density": 50,
        "dont_filter_internal_bridges": "limited"
    })));

    assert!(gcode.contains(";PRINT_PATH:internal_bridge:"));
    assert!(gcode.contains(";EXTRUSION:print:internal_bridge:"));
}

#[test]
fn nofilter_emits_internal_bridge_gcode_for_small_candidate() {
    let gcode = small_internal_bridge_filter_gcode(options(json!({
        "internal_bridge_density": 50,
        "dont_filter_internal_bridges": "nofilter"
    })));

    assert!(gcode.contains(";PRINT_PATH:internal_bridge:"));
    assert!(gcode.contains(";EXTRUSION:print:internal_bridge:"));
}

fn internal_bridge_density_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn small_internal_bridge_filter_gcode(options: SliceOptions) -> String {
    let pipeline = crate::pipeline::test_support::contour_layers_pipeline(
        &options,
        vec![crate::Contour::new(vec![
            crate::Point2::new(0.0, 0.0),
            crate::Point2::new(1.5, 0.0),
            crate::Point2::new(1.5, 1.5),
            crate::Point2::new(0.0, 1.5),
        ])],
        3,
    );
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
