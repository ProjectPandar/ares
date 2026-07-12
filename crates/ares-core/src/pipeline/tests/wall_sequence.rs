use super::*;
use serde_json::json;

#[test]
fn wall_sequence_changes_perimeter_and_print_path_order_in_gcode() {
    let inner_outer: SliceOptions = serde_json::from_value(json!({
        "wall_sequence": "inner wall/outer wall",
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let outer_inner: SliceOptions = serde_json::from_value(json!({
        "wall_sequence": "outer wall/inner wall",
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let inner_outer_gcode = String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&inner_outer), &inner_outer).unwrap(),
    )
    .unwrap();
    let outer_inner_gcode = String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&outer_inner), &outer_inner).unwrap(),
    )
    .unwrap();

    let inner_outer_internal_perimeter = inner_outer_gcode
        .find(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708")
        .expect("inner/outer internal perimeter marker");
    let inner_outer_external_perimeter = inner_outer_gcode
        .find(";PERIMETER:external:0,0 -> 4,0")
        .expect("inner/outer external perimeter marker");
    let outer_inner_external_perimeter = outer_inner_gcode
        .find(";PERIMETER:external:0,0 -> 4,0")
        .expect("outer/inner external perimeter marker");
    let outer_inner_internal_perimeter = outer_inner_gcode
        .find(";PERIMETER:internal:0.35708,0.35708 -> 3.64292,0.35708")
        .expect("outer/inner internal perimeter marker");
    let inner_outer_internal_print_path = inner_outer_gcode
        .find(";PRINT_PATH:internal_perimeter:0.75708,0.75708 -> 3.24292,0.75708")
        .expect("inner/outer internal print path marker");
    let inner_outer_external_print_path = inner_outer_gcode
        .find(";PRINT_PATH:external_perimeter:0,0 -> 4,0")
        .expect("inner/outer external print path marker");
    let outer_inner_external_print_path = outer_inner_gcode
        .find(";PRINT_PATH:external_perimeter:0,0 -> 4,0")
        .expect("outer/inner external print path marker");
    let outer_inner_internal_print_path = outer_inner_gcode
        .find(";PRINT_PATH:internal_perimeter:0.35708,0.35708 -> 3.64292,0.35708")
        .expect("outer/inner internal print path marker");

    assert!(inner_outer_internal_perimeter < inner_outer_external_perimeter);
    assert!(outer_inner_external_perimeter < outer_inner_internal_perimeter);
    assert!(inner_outer_internal_print_path < inner_outer_external_print_path);
    assert!(outer_inner_external_print_path < outer_inner_internal_print_path);
    assert_ne!(inner_outer_gcode, outer_inner_gcode);
}
