use super::*;
use serde_json::json;

#[test]
fn precise_outer_wall_false_changes_inner_outer_gcode_geometry() {
    let enabled = options(json!({ "precise_outer_wall": true }));
    let disabled = options(json!({ "precise_outer_wall": false }));

    let enabled_gcode = gcode_for(&enabled);
    let disabled_gcode = gcode_for(&disabled);

    assert!(enabled_gcode.contains(";PERIMETER:internal:0.80708,0.80708 -> 3.19292,0.80708"));
    assert!(enabled_gcode.contains(";PERIMETER:internal:0.45,0.45 -> 3.55,0.45"));
    assert!(disabled_gcode.contains(";PERIMETER:internal:0.764159,0.764159 -> 3.235841,0.764159"));
    assert!(disabled_gcode.contains(";PERIMETER:internal:0.40708,0.40708 -> 3.59292,0.40708"));
    assert_ne!(enabled_gcode, disabled_gcode);
}

#[test]
fn non_inner_outer_sequences_ignore_precise_outer_wall_in_gcode() {
    for sequence in ["outer wall/inner wall", "inner-outer-inner wall"] {
        let enabled = options(json!({
            "wall_sequence": sequence,
            "precise_outer_wall": true
        }));
        let disabled = options(json!({
            "wall_sequence": sequence,
            "precise_outer_wall": false
        }));

        assert_eq!(gcode_for(&enabled), gcode_for(&disabled));
    }
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "outer_wall_line_width": 0.5,
        "inner_wall_line_width": 0.4,
        "wall_sequence": "inner wall/outer wall",
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
