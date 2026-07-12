use super::*;
use serde_json::json;

#[test]
fn enabled_wipe_before_external_loop_moves_inward_and_returns_before_external_extrusion() {
    let disabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "wipe_before_external_loop": false,
        "gcode_comments": true
    }))
    .unwrap();
    let enabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "wipe_before_external_loop": true,
        "gcode_comments": true
    }))
    .unwrap();

    assert_ordered(
        &enabled,
        &[
            "G1 X0 Y0 F7200 ; travel",
            "G1 X0.141 Y0.141 F3600 ; wipe before external loop",
            "G1 X0 Y0 F3600 ; wipe before external loop",
            "G1 X4 Y0 E0.11876 ; extrude",
        ],
    );
    assert!(enabled.contains(";MOVE:print:external_perimeter:0,0"));
    assert!(enabled.contains(";EXTRUSION:print:external_perimeter:4,0:0.118765"));
    assert_ne!(enabled, disabled);
}

#[test]
fn disabled_wipe_before_external_loop_preserves_external_perimeter_gcode() {
    let absent = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "gcode_comments": true
    }))
    .unwrap();
    let disabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "wipe_before_external_loop": false,
        "gcode_comments": true
    }))
    .unwrap();

    assert!(absent.contains("G1 X4 Y0 E0.11876 ; extrude"));
    assert!(disabled.contains("G1 X4 Y0 E0.11876 ; extrude"));
    assert!(!disabled.contains("wipe before external loop"));
}

#[test]
fn enabled_wipe_before_external_loop_without_internal_perimeter_preserves_output() {
    let disabled = rectangular_gcode(json!({
        "wall_loops": 1,
        "wipe_before_external_loop": false,
        "gcode_comments": true
    }))
    .unwrap();
    let enabled = rectangular_gcode(json!({
        "wall_loops": 1,
        "wipe_before_external_loop": true,
        "gcode_comments": true
    }))
    .unwrap();

    assert_eq!(enabled, disabled);
    assert!(!enabled.contains("wipe before external loop"));
}

#[test]
fn wipe_before_external_loop_rejects_non_boolean_values() {
    for value in [json!("true"), json!(1), json!([true])] {
        let err = rectangular_gcode(json!({
            "wipe_before_external_loop": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("wipe_before_external_loop"));
    }
}

fn rectangular_gcode(extra: serde_json::Value) -> Result<String, SliceError> {
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
    let options: SliceOptions = serde_json::from_value(value).unwrap();
    Ok(String::from_utf8(crate::gcode::format_gcode(
        &rectangular_pipeline(&options),
        &options,
    )?)
    .unwrap())
}

fn assert_ordered(output: &str, expected: &[&str]) {
    let lines = output.lines().collect::<Vec<_>>();
    let mut start = 0;
    for expected_line in expected {
        let offset = lines[start..]
            .iter()
            .position(|line| line == expected_line)
            .unwrap_or_else(|| panic!("{expected_line} missing after {start}\n{output}"));
        start += offset + 1;
    }
}
