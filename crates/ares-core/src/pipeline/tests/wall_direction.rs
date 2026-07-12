use super::*;
use serde_json::json;

#[test]
fn wall_direction_changes_perimeter_and_print_path_order_in_gcode() {
    let ccw: SliceOptions = serde_json::from_value(json!({
        "wall_direction": "ccw",
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let cw: SliceOptions = serde_json::from_value(json!({
        "wall_direction": "cw",
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let ccw_gcode =
        String::from_utf8(crate::gcode::format_gcode(&rectangular_pipeline(&ccw), &ccw).unwrap())
            .unwrap();
    let cw_gcode =
        String::from_utf8(crate::gcode::format_gcode(&rectangular_pipeline(&cw), &cw).unwrap())
            .unwrap();

    assert!(ccw_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert!(cw_gcode.contains(";PERIMETER:external:0,4 -> 4,4 -> 4,0 -> 0,0"));
    assert!(ccw_gcode.contains(";PRINT_PATH:external_perimeter:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert!(cw_gcode.contains(";PRINT_PATH:external_perimeter:0,4 -> 4,4 -> 4,0 -> 0,0"));
    assert_ne!(ccw_gcode, cw_gcode);
}

#[test]
fn back_seam_position_changes_perimeter_and_print_path_start_in_gcode() {
    let aligned: SliceOptions = serde_json::from_value(json!({
        "seam_position": "aligned",
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let back: SliceOptions = serde_json::from_value(json!({
        "seam_position": "back",
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let aligned_gcode = String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&aligned), &aligned).unwrap(),
    )
    .unwrap();
    let back_gcode =
        String::from_utf8(crate::gcode::format_gcode(&rectangular_pipeline(&back), &back).unwrap())
            .unwrap();

    assert!(aligned_gcode.contains(";PERIMETER:external:0,0 -> 4,0 -> 4,4 -> 0,4"));
    assert!(back_gcode.contains(";PERIMETER:external:4,4 -> 0,4 -> 0,0 -> 4,0"));
    assert!(back_gcode.contains(";PRINT_PATH:external_perimeter:4,4 -> 0,4 -> 0,0 -> 4,0"));
    assert_ne!(aligned_gcode, back_gcode);
}

#[test]
fn seam_gap_shortens_external_perimeter_closing_move_in_gcode() {
    let baseline = rectangular_gcode(json!({
        "seam_gap": 0,
        "wall_loops": 1
    }));
    let clipped = rectangular_gcode(json!({
        "seam_gap": 1.0,
        "wall_loops": 1
    }));

    assert!(has_line(&baseline, ";MOVE:print:external_perimeter:0,0"));
    assert!(has_line_start(
        &baseline,
        ";EXTRUSION:print:external_perimeter:0,0:"
    ));
    assert!(has_line(&clipped, ";MOVE:print:external_perimeter:0,1"));
    assert!(!has_line(&clipped, ";MOVE:print:external_perimeter:0,0"));
    assert!(has_line_start(
        &clipped,
        ";EXTRUSION:print:external_perimeter:0,1:"
    ));
    assert_ne!(
        perimeter_lines(&baseline, "external_perimeter"),
        perimeter_lines(&clipped, "external_perimeter")
    );
}

#[test]
fn seam_gap_percent_uses_external_width_for_closing_move() {
    let clipped = rectangular_gcode(json!({
        "line_width": 0.4,
        "seam_gap": "50%",
        "wall_loops": 1
    }));

    assert!(has_line(&clipped, ";MOVE:print:external_perimeter:0,0.2"));
    assert!(!has_line(&clipped, ";MOVE:print:external_perimeter:0,0"));
}

#[test]
fn omitted_seam_gap_defaults_to_ten_percent_closing_move_in_gcode() {
    let clipped = rectangular_gcode(json!({
        "line_width": 0.4,
        "wall_loops": 1
    }));

    assert!(has_line(&clipped, ";MOVE:print:external_perimeter:0,0.04"));
    assert!(!has_line(&clipped, ";MOVE:print:external_perimeter:0,0"));
}

#[test]
fn zero_seam_gap_preserves_original_external_perimeter_closure() {
    let zero = rectangular_gcode(json!({
        "seam_gap": 0,
        "wall_loops": 1
    }));

    assert!(has_line(&zero, ";MOVE:print:external_perimeter:0,0"));
    assert!(has_line_start(
        &zero,
        ";EXTRUSION:print:external_perimeter:0,0:"
    ));
    assert!(!has_line(&zero, ";MOVE:print:external_perimeter:0,0.04"));
}

#[test]
fn seam_gap_shortens_internal_and_overhang_perimeter_closures() {
    let internal = rectangular_gcode(json!({
        "seam_gap": 1.0,
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall"
    }));
    let overhang = unsupported_second_layer_gcode(json!({
        "seam_gap": 1.0,
        "wall_loops": 1
    }));

    assert!(has_line(
        &internal,
        ";MOVE:print:internal_perimeter:0.35708,1.35708"
    ));
    assert!(has_line(&overhang, ";MOVE:print:overhang_perimeter:10,1"));
}

#[test]
fn seam_gap_does_not_clip_skirt_or_brim_closures() {
    let baseline = rectangular_gcode(json!({
        "seam_gap": 0,
        "wall_loops": 1,
        "skirt_loops": 1,
        "brim_width": 0.4
    }));
    let clipped = rectangular_gcode(json!({
        "seam_gap": 1.0,
        "wall_loops": 1,
        "skirt_loops": 1,
        "brim_width": 0.4
    }));

    assert!(
        role_moves(&clipped, "skirt")
            .iter()
            .any(|line| { line.starts_with(";MOVE:print:skirt:") })
    );
    assert!(
        role_moves(&clipped, "brim")
            .iter()
            .any(|line| { line.starts_with(";MOVE:print:brim:") })
    );
    assert_eq!(
        role_moves(&baseline, "skirt"),
        role_moves(&clipped, "skirt")
    );
    assert_eq!(role_moves(&baseline, "brim"), role_moves(&clipped, "brim"));
}

#[test]
fn oversized_seam_gap_omits_closing_perimeter_move() {
    let clipped = rectangular_gcode(json!({
        "seam_gap": 10.0,
        "wall_loops": 1
    }));

    assert!(!has_line(&clipped, ";MOVE:print:external_perimeter:0,0"));
    assert!(has_line(&clipped, ";MOVE:print:external_perimeter:0,4"));
}

fn has_line(gcode: &str, expected: &str) -> bool {
    gcode.lines().any(|line| line == expected)
}

fn has_line_start(gcode: &str, expected: &str) -> bool {
    gcode.lines().any(|line| line.starts_with(expected))
}

fn perimeter_lines<'a>(gcode: &'a str, role: &str) -> Vec<&'a str> {
    gcode
        .lines()
        .filter(|line| {
            line.starts_with(&format!(";MOVE:print:{role}:"))
                || line.starts_with(&format!(";EXTRUSION:print:{role}:"))
        })
        .collect()
}

fn role_moves<'a>(gcode: &'a str, role: &str) -> Vec<&'a str> {
    gcode
        .lines()
        .filter(|line| line.starts_with(&format!(";MOVE:print:{role}:")))
        .collect()
}

fn rectangular_gcode(extra: serde_json::Value) -> String {
    let options = seam_gap_options(extra);
    String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&options), &options).unwrap(),
    )
    .unwrap()
}

fn unsupported_second_layer_gcode(extra: serde_json::Value) -> String {
    let options = seam_gap_options(extra);
    String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&options),
            &options,
        )
        .unwrap(),
    )
    .unwrap()
}

fn seam_gap_options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
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
