use super::*;
use serde_json::json;

#[test]
fn enabled_external_fuzzy_skin_changes_external_perimeter_gcode() {
    let disabled = options(json!({ "fuzzy_skin": "disabled_fuzzy" }));
    let enabled = options(json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3
    }));

    let disabled_gcode = gcode_for(&disabled);
    let enabled_gcode = gcode_for(&enabled);

    assert!(disabled_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert!(enabled_gcode.contains(";PERIMETER:external:"));
    assert!(!enabled_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert_ne!(enabled_gcode, disabled_gcode);
}

#[test]
fn allwalls_fuzzy_skin_changes_internal_perimeter_gcode() {
    let disabled = options(json!({
        "wall_loops": 2,
        "fuzzy_skin": "disabled_fuzzy",
        "gcode_comments": true
    }));
    let allwalls = options(json!({
        "wall_loops": 2,
        "fuzzy_skin": "allwalls",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3,
        "gcode_comments": true
    }));

    let disabled_gcode = gcode_for(&disabled);
    let allwalls_gcode = gcode_for(&allwalls);
    let disabled_internal = internal_diagnostic_lines(&disabled_gcode);
    let allwalls_internal = internal_diagnostic_lines(&allwalls_gcode);

    assert!(!disabled_internal.is_empty());
    assert!(
        allwalls_internal
            .iter()
            .any(|line| line.starts_with(";PERIMETER:internal:"))
    );
    assert!(
        allwalls_internal
            .iter()
            .any(|line| line.starts_with(";PRINT_PATH:internal_perimeter:"))
    );
    assert_ne!(allwalls_internal, disabled_internal);
    assert!(disabled_gcode.contains(";PERIMETER:internal:0.4,0.4 -> 3.6,0.4"));
    assert!(!allwalls_gcode.contains(";PERIMETER:internal:0.4,0.4 -> 3.6,0.4"));
}

#[test]
fn perlin_fuzzy_skin_changes_external_perimeter_gcode() {
    let disabled = options(json!({ "fuzzy_skin": "disabled_fuzzy" }));
    let perlin = options(json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_noise_type": "perlin",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3,
        "fuzzy_skin_scale": 1.0,
        "fuzzy_skin_octaves": 4,
        "fuzzy_skin_persistence": 0.5
    }));

    let disabled_gcode = gcode_for(&disabled);
    let perlin_gcode = gcode_for(&perlin);

    assert!(disabled_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert!(perlin_gcode.contains(";PERIMETER:external:"));
    assert!(!perlin_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert_ne!(perlin_gcode, disabled_gcode);
}

#[test]
fn fuzzy_skin_default_preserves_explicit_disabled_geometry() {
    let default = options(json!({}));
    let explicit_disabled = options(json!({ "fuzzy_skin": "disabled_fuzzy" }));

    assert_eq!(
        path_lines(&gcode_for(&default)),
        path_lines(&gcode_for(&explicit_disabled))
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn gcode_for(options: &SliceOptions) -> String {
    String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::rectangular_pipeline(options),
            options,
        )
        .unwrap(),
    )
    .unwrap()
}

fn path_lines(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| {
            line.starts_with(";PERIMETER:")
                || line.starts_with(";PRINT_PATH:")
                || line.starts_with(";MOVE:")
                || line.starts_with("G1 X")
        })
        .collect()
}

fn internal_diagnostic_lines(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| {
            line.starts_with(";PERIMETER:internal:")
                || line.starts_with(";PRINT_PATH:internal_perimeter:")
        })
        .collect()
}
