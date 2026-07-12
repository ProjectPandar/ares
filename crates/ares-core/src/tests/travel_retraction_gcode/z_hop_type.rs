use super::*;

use serde_json::{Value, json};

use crate::PrintPathRole;

fn role_output(extra: Value) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop": 0.4,
        "travel_slope": 45,
        "gcode_comments": true
    });
    let options = options_json.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline = crate::pipeline::layer_change_test_support::role_layers_pipeline(
        &options,
        vec![vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::ExternalPerimeter,
        ]],
    );
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

fn zero_distance_output(extra: Value) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.0,
        "z_hop": 0.4,
        "z_hop_types": ["Slope Lift"],
        "gcode_comments": true
    });
    let options = options_json.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline =
        crate::pipeline::layer_change_test_support::zero_distance_travel_after_print_pipeline(
            &options,
        );
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

#[test]
fn normal_lift_preserves_separate_vertical_lift_before_ordinary_travel() {
    let output = role_output(json!({
        "z_hop_types": ["Normal Lift"]
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let travel = line_index(layer, "G1 X2 Y0 F7200 ; travel");
    let retract = previous_line_index(layer, travel, "G1 E-0.8 F1800 ; retract");
    let lift = previous_line_index(layer, travel, "G1 Z0.6 F7200 ; lift Z");
    let restore = next_line_index(layer, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(layer, restore, "G1 E0.8 F1800 ; unretract");

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn default_slope_lift_splits_lift_into_ordinary_travel() {
    let output = role_output(json!({})).unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let slope = next_line_index(layer, retract, "G1 X0.4 Y0 Z0.6 F7200 ; travel");
    let travel = next_line_index(layer, slope, "G1 X2 Y0 F7200 ; travel");
    let restore = next_line_index(layer, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(layer, restore, "G1 E0.8 F1800 ; unretract");

    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(retract < slope);
    assert!(slope < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn explicit_slope_lift_uses_travel_slope_angle() {
    let output = role_output(json!({
        "z_hop_types": ["Slope Lift"],
        "travel_slope": 45
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let slope = next_line_index(layer, retract, "G1 X0.4 Y0 Z0.6 F7200 ; travel");
    let travel = next_line_index(layer, slope, "G1 X2 Y0 F7200 ; travel");

    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(retract < slope);
    assert!(slope < travel);
}

#[test]
fn too_short_slope_lift_emits_single_raised_xyz_travel() {
    let output = role_output(json!({
        "z_hop_types": ["Slope Lift"],
        "travel_slope": 10
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let raised_travel = next_line_index(layer, retract, "G1 X2 Y0 Z0.6 F7200 ; travel");
    let restore = next_line_index(layer, raised_travel, "G1 Z0.2 F7200 ; restore layer Z");
    let raised_count = layer
        .lines()
        .filter(|line| *line == "G1 X2 Y0 Z0.6 F7200 ; travel")
        .count();

    assert_eq!(raised_count, 1);
    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(!layer.lines().any(|line| line == "G1 X2 Y0 F7200 ; travel"));
    assert!(raised_travel < restore);
}

#[test]
fn zero_distance_slope_lift_emits_one_raised_xyz_travel_and_keeps_metadata() {
    let output = zero_distance_output(json!({})).unwrap();
    let layer = layer_section(&output, 0);
    let target = "G1 X1 Y0 Z0.6 F7200 ; travel";
    let raised_count = layer.lines().filter(|line| *line == target).count();
    let speed = line_index(layer, ";SPEED:travel:external_perimeter:1,0:7200");
    let extrusion = next_line_index(layer, speed, ";EXTRUSION:travel:external_perimeter:1,0:");
    let move_meta = next_line_index(layer, extrusion, ";MOVE:travel:external_perimeter:1,0");
    let raised = next_line_index(layer, move_meta, target);

    assert_eq!(raised_count, 1);
    assert!(!layer.lines().any(|line| line == "G1 X1 Y0 F7200 ; travel"));
    assert!(speed < extrusion);
    assert!(extrusion < move_meta);
    assert!(move_meta < raised);
}

#[test]
fn spiral_lift_emits_linearized_spiral_before_ordinary_travel() {
    let output = role_output(json!({
        "z_hop_types": ["Spiral Lift"],
        "resolution": 0.01
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let comment = next_line_index(layer, retract, ";spiral lift Z");
    let feedrate = next_line_index(layer, comment, "G1 F7200");
    let first_spiral = next_line_index(layer, feedrate, "G1 X0.037 Y0.007 Z0.225");
    let second_spiral = next_line_index(layer, first_spiral, "G1 X0.068 Y0.028 Z0.25");
    let final_spiral = next_line_index(layer, second_spiral, "G1 X0 Y0 Z0.6");
    let travel = next_line_index(layer, final_spiral, "G1 X2 Y0 F7200 ; travel");
    let restore = next_line_index(layer, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(layer, restore, "G1 E0.8 F1800 ; unretract");

    assert_eq!(spiral_segment_count(layer), 16);
    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(retract < comment);
    assert!(comment < feedrate);
    assert!(first_spiral < second_spiral);
    assert!(second_spiral < final_spiral);
    assert!(final_spiral < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn spiral_lift_resolution_controls_segment_count() {
    let output = role_output(json!({
        "z_hop_types": ["Spiral Lift"],
        "resolution": 0.02
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    assert_eq!(spiral_segment_count(layer), 8);
    assert!(layer.lines().any(|line| line == "G1 X0.068 Y0.028 Z0.25"));
    assert!(layer.lines().any(|line| line == "G1 X0 Y0 Z0.6"));
}

#[test]
fn spiral_lift_omits_spiral_comment_when_gcode_comments_are_disabled() {
    let output = role_output(json!({
        "z_hop_types": ["Spiral Lift"],
        "gcode_comments": false
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    assert_eq!(spiral_segment_count(layer), 16);
    assert!(!layer.lines().any(|line| line.contains("spiral lift Z")));
}

#[test]
fn auto_lift_uses_slope_lift_without_overhang_crossing_data() {
    let output = role_output(json!({
        "z_hop_types": ["Auto Lift"]
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let slope = next_line_index(layer, retract, "G1 X0.4 Y0 Z0.6 F7200 ; travel");
    let travel = next_line_index(layer, slope, "G1 X2 Y0 F7200 ; travel");
    let restore = next_line_index(layer, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(layer, restore, "G1 E0.8 F1800 ; unretract");

    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(!layer.lines().any(|line| line.contains("spiral lift Z")));
    assert!(retract < slope);
    assert!(slope < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn filament_z_hop_type_overrides_default_slope_lift() {
    let output = role_output(json!({
        "filament_z_hop_types": ["Normal Lift", "Slope Lift"]
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let travel = line_index(layer, "G1 X2 Y0 F7200 ; travel");
    let lift = previous_line_index(layer, travel, "G1 Z0.6 F7200 ; lift Z");

    assert!(lift < travel);
    assert!(
        !layer
            .lines()
            .any(|line| line == "G1 X0.4 Y0 Z0.6 F7200 ; travel")
    );
}

#[test]
fn filament_spiral_lift_overrides_unprefixed_normal_lift() {
    let output = role_output(json!({
        "z_hop_types": ["Normal Lift"],
        "filament_z_hop_types": ["Spiral Lift"],
        "resolution": 0.01
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    let retract = line_index(layer, "G1 E-0.8 F1800 ; retract");
    let comment = next_line_index(layer, retract, ";spiral lift Z");
    let final_segment = next_line_index(layer, comment, "G1 X0 Y0 Z0.6");
    let travel = next_line_index(layer, final_segment, "G1 X2 Y0 F7200 ; travel");

    assert_eq!(spiral_segment_count(layer), 16);
    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(comment < final_segment);
    assert!(final_segment < travel);
}

#[test]
fn nil_filament_z_hop_type_falls_back_to_unprefixed_slope_lift() {
    let output = role_output(json!({
        "z_hop_types": ["Slope Lift"],
        "filament_z_hop_types": "nil,Normal Lift"
    }))
    .unwrap();
    let layer = layer_section(&output, 0);

    assert!(
        layer
            .lines()
            .any(|line| line == "G1 X0.4 Y0 Z0.6 F7200 ; travel")
    );
    assert!(!layer.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
}

#[test]
fn zero_distance_spiral_lift_emits_one_raised_xyz_travel_without_static_spiral() {
    let output = zero_distance_output(json!({
        "z_hop_types": ["Spiral Lift"],
        "travel_slope": 45,
        "resolution": 0.01
    }))
    .unwrap();
    let layer = layer_section(&output, 0);
    let target = "G1 X1 Y0 Z0.6 F7200 ; travel";
    let raised_count = layer.lines().filter(|line| *line == target).count();
    let speed = line_index(layer, ";SPEED:travel:external_perimeter:1,0:7200");
    let extrusion = next_line_index(layer, speed, ";EXTRUSION:travel:external_perimeter:1,0:");
    let move_meta = next_line_index(layer, extrusion, ";MOVE:travel:external_perimeter:1,0");
    let raised = next_line_index(layer, move_meta, target);

    assert_eq!(raised_count, 1);
    assert_eq!(spiral_segment_count(layer), 0);
    assert!(!layer.lines().any(|line| line.contains("spiral lift Z")));
    assert!(!layer.lines().any(|line| line == "G1 F7200"));
    assert!(!layer.lines().any(|line| line == "G1 X1 Y0 F7200 ; travel"));
    assert!(speed < extrusion);
    assert!(extrusion < move_meta);
    assert!(move_meta < raised);
}

#[test]
fn invalid_z_hop_type_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("z_hop_types", json!([])),
        ("z_hop_types", json!("Bad Lift")),
        ("z_hop_types", json!("")),
        ("z_hop_types", json!("Slope Lift,Bad Lift")),
        ("z_hop_types", json!(["Slope Lift", "Bad Lift"])),
        ("z_hop_types", json!([1])),
        ("filament_z_hop_types", json!([])),
        ("filament_z_hop_types", json!("Bad Lift")),
        ("filament_z_hop_types", json!("Slope Lift,")),
        ("filament_z_hop_types", json!(["Slope Lift", "Bad Lift"])),
        ("filament_z_hop_types", json!([null, "Bad Lift"])),
        ("filament_z_hop_types", json!([1])),
    ] {
        let err = role_output(json!({ key: value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key} missing from {err}");
    }
}

#[test]
fn invalid_travel_slope_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(0),
        json!(91),
        json!("inf"),
        json!("bad"),
        json!([45, "bad"]),
        json!([45, 0]),
    ] {
        let err = role_output(json!({
            "z_hop_types": ["Slope Lift"],
            "travel_slope": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("travel_slope"),
            "travel_slope missing from {err}"
        );
    }
}

#[test]
fn invalid_resolution_values_are_rejected_with_option_key() {
    for value in [
        json!(-0.1),
        json!("bad"),
        json!("inf"),
        json!(true),
        json!([0.01]),
    ] {
        let err = role_output(json!({
            "z_hop_types": ["Spiral Lift"],
            "resolution": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("resolution"),
            "resolution missing from {err}"
        );
    }
}

fn spiral_segment_count(section: &str) -> usize {
    section
        .lines()
        .filter(|line| line.starts_with("G1 X") && line.contains(" Z") && !line.contains(" F"))
        .count()
}
