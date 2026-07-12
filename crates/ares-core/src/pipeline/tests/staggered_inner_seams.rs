use super::*;
use serde_json::json;

#[test]
fn staggered_inner_seams_changes_internal_perimeter_path_start_in_gcode() {
    let disabled = gcode(json!({
        "staggered_inner_seams": false,
        "seam_position": "back",
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "seam_gap": 0
    }));
    let enabled = gcode(json!({
        "staggered_inner_seams": true,
        "seam_position": "back",
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "seam_gap": 0
    }));

    assert!(disabled.contains(
        ";PERIMETER:internal:3.64292,3.64292 -> 0.35708,3.64292 -> 0.35708,0.35708 -> 3.64292,0.35708"
    ));
    assert!(enabled.contains(
        ";PERIMETER:internal:3.24292,3.64292 -> 0.35708,3.64292 -> 0.35708,0.35708 -> 3.64292,0.35708 -> 3.64292,3.64292"
    ));
    assert!(enabled.contains(
        ";PRINT_PATH:internal_perimeter:3.24292,3.64292 -> 0.35708,3.64292 -> 0.35708,0.35708 -> 3.64292,0.35708 -> 3.64292,3.64292"
    ));
    assert!(enabled.contains(";MOVE:travel:internal_perimeter:3.24292,3.64292"));
    assert!(enabled.contains(";MOVE:print:internal_perimeter:3.24292,3.64292"));
    assert!(enabled.contains(";PERIMETER:external:4,4 -> 0,4 -> 0,0 -> 4,0"));
    assert_ne!(disabled, enabled);
}

fn gcode(extra: serde_json::Value) -> String {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    let options: SliceOptions = serde_json::from_value(value).unwrap();
    String::from_utf8(
        crate::gcode::format_gcode(&rectangular_pipeline(&options), &options).unwrap(),
    )
    .unwrap()
}
