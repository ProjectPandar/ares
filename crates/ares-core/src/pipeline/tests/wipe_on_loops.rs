use super::*;
use serde_json::json;

#[test]
fn enabled_wipe_on_loops_moves_inward_after_external_loop_before_internal_travel() {
    let disabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "seam_gap": 0,
        "wipe_on_loops": false,
        "gcode_comments": true
    }))
    .unwrap();
    let enabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "outer_wall_speed": 60,
        "initial_layer_speed": 60,
        "seam_gap": 0,
        "wipe_on_loops": true,
        "gcode_comments": true
    }))
    .unwrap();

    assert_ordered(
        &enabled,
        &[
            "G1 X0 Y0 E0.11877 ; extrude",
            "G1 X0.069 Y0.04 F3600 ; move inwards before travel",
            "G1 X0.357 Y0.357 F7200 ; travel",
        ],
    );
    assert!(!enabled.contains("wipe before external loop"));
    assert_ne!(enabled, disabled);
}

#[test]
fn missing_or_disabled_wipe_on_loops_preserves_output() {
    let absent = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "seam_gap": 0,
        "gcode_comments": true
    }))
    .unwrap();
    let disabled = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "seam_gap": 0,
        "wipe_on_loops": false,
        "gcode_comments": true
    }))
    .unwrap();

    assert!(!absent.contains("move inwards before travel"));
    assert!(!disabled.contains("move inwards before travel"));
    assert!(absent.contains("G1 X0 Y0 E0.11877 ; extrude"));
    assert!(disabled.contains("G1 X0 Y0 E0.11877 ; extrude"));
}

#[test]
fn wipe_on_loops_skips_without_multiple_wall_loops_or_external_closing_move() {
    let one_wall = rectangular_gcode(json!({
        "wall_loops": 1,
        "seam_gap": 0,
        "wipe_on_loops": true,
        "gcode_comments": true
    }))
    .unwrap();
    let no_closing_move = rectangular_gcode(json!({
        "wall_loops": 2,
        "wall_sequence": "outer wall/inner wall",
        "seam_gap": 10.0,
        "wipe_on_loops": true,
        "gcode_comments": true
    }))
    .unwrap();

    assert!(!one_wall.contains("move inwards before travel"));
    assert!(!no_closing_move.contains("move inwards before travel"));
}

#[test]
fn wipe_on_loops_rejects_non_boolean_values() {
    for value in [json!("true"), json!(1), json!([true])] {
        let err = rectangular_gcode(json!({
            "wipe_on_loops": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("wipe_on_loops"));
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
