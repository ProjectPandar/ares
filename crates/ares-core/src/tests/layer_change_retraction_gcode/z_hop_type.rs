use super::*;

use serde_json::{Value, json};

fn role_output(extra: Value) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "z_hop": 0.4,
        "travel_slope": 45,
        "gcode_comments": true
    });
    let options = options_json.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline =
        crate::pipeline::layer_change_test_support::pending_travel_layer_boundary_pipeline(
            &options,
        );
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

fn print_first_output(extra: Value) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "z_hop": 0.4,
        "z_hop_types": ["Slope Lift"],
        "travel_slope": 45,
        "gcode_comments": true
    });
    let options = options_json.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline =
        crate::pipeline::layer_change_test_support::print_first_after_layer_change_pipeline(
            &options,
        );
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

#[test]
fn normal_lift_preserves_vertical_layer_change_lift() {
    let output = role_output(json!({
        "z_hop_types": ["Normal Lift"]
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let lift = line_index(second, "G1 Z0.8 F7200 ; lift Z");
    let restore = line_index(second, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(retract < z);
    assert!(z < lift);
    assert!(lift < restore);
    assert!(restore < unretract);
    assert!(unretract < first_print);
}

#[test]
fn default_slope_lift_consumes_next_layer_travel() {
    let output = role_output(json!({})).unwrap();
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = next_line_index(second, retract, "G1 Z0.4 F7200 ; move to layer Z");
    let slope = next_line_index(second, z, "G1 X1.6 Y0 Z0.8 F7200 ; travel");
    let travel = next_line_index(second, slope, "G1 X0 Y0 F7200 ; travel");
    let restore = next_line_index(second, travel, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = next_line_index(second, restore, "G1 E0.8 F1800 ; unretract");

    assert!(!second.lines().any(|line| line == "G1 Z0.8 F7200 ; lift Z"));
    assert!(retract < z);
    assert!(z < slope);
    assert!(slope < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn too_short_layer_change_slope_lift_emits_single_raised_travel() {
    let output = role_output(json!({
        "z_hop_types": ["Slope Lift"],
        "travel_slope": 10
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let raised = next_line_index(second, retract, "G1 X0 Y0 Z0.8 F7200 ; travel");
    let restore = next_line_index(second, raised, "G1 Z0.4 F7200 ; restore layer Z");

    assert_eq!(
        second
            .lines()
            .filter(|line| *line == "G1 X0 Y0 Z0.8 F7200 ; travel")
            .count(),
        1
    );
    assert!(!second.lines().any(|line| line == "G1 Z0.8 F7200 ; lift Z"));
    assert!(!second.lines().any(|line| line == "G1 X0 Y0 F7200 ; travel"));
    assert!(raised < restore);
}

#[test]
fn pending_slope_lift_falls_back_to_vertical_lift_before_print_without_travel() {
    let output = print_first_output(json!({})).unwrap();
    let second = layer_section(&output, 1);

    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let lift = next_line_index(second, z, "G1 Z0.8 F7200 ; lift Z");
    let restore = next_line_index(second, lift, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = next_line_index(second, restore, "G1 E0.8 F1800 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(
        !second
            .lines()
            .any(|line| line.contains(" Z0.8 F7200 ; travel"))
    );
    assert!(z < lift);
    assert!(lift < restore);
    assert!(restore < unretract);
    assert!(unretract < first_print);
}

#[test]
fn spiral_lift_consumes_pending_layer_change_travel() {
    let output = role_output(json!({
        "z_hop_types": ["Spiral Lift"],
        "resolution": 0.01
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = next_line_index(second, retract, "G1 Z0.4 F7200 ; move to layer Z");
    let comment = next_line_index(second, z, ";spiral lift Z");
    let feedrate = next_line_index(second, comment, "G1 F7200");
    let first_spiral = next_line_index(second, feedrate, "G1 X1.963 Y-0.007 Z0.425");
    let final_spiral = next_line_index(second, first_spiral, "G1 X2 Y0 Z0.8");
    let travel = next_line_index(second, final_spiral, "G1 X0 Y0 F7200 ; travel");
    let restore = next_line_index(second, travel, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = next_line_index(second, restore, "G1 E0.8 F1800 ; unretract");

    assert_eq!(spiral_segment_count(second), 16);
    assert!(!second.lines().any(|line| line == "G1 Z0.8 F7200 ; lift Z"));
    assert!(retract < z);
    assert!(comment < first_spiral);
    assert!(feedrate < first_spiral);
    assert!(first_spiral < final_spiral);
    assert!(final_spiral < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn auto_lift_forces_spiral_layer_change_lift() {
    let output = role_output(json!({
        "z_hop_types": ["Auto Lift"],
        "resolution": 0.01
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = next_line_index(second, retract, "G1 Z0.4 F7200 ; move to layer Z");
    let comment = next_line_index(second, z, ";spiral lift Z");
    let feedrate = next_line_index(second, comment, "G1 F7200");
    let first_spiral = next_line_index(second, feedrate, "G1 X1.963 Y-0.007 Z0.425");
    let final_spiral = next_line_index(second, first_spiral, "G1 X2 Y0 Z0.8");
    let travel = next_line_index(second, final_spiral, "G1 X0 Y0 F7200 ; travel");
    let restore = next_line_index(second, travel, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = next_line_index(second, restore, "G1 E0.8 F1800 ; unretract");

    assert_eq!(spiral_segment_count(second), 16);
    assert!(!second.lines().any(|line| line == "G1 Z0.8 F7200 ; lift Z"));
    assert!(retract < z);
    assert!(comment < first_spiral);
    assert!(feedrate < first_spiral);
    assert!(first_spiral < final_spiral);
    assert!(final_spiral < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[test]
fn filament_spiral_lift_overrides_unprefixed_normal_layer_change_lift() {
    let output = role_output(json!({
        "z_hop_types": ["Normal Lift"],
        "filament_z_hop_types": ["Spiral Lift"],
        "resolution": 0.01
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let comment = next_line_index(second, z, ";spiral lift Z");
    let final_segment = next_line_index(second, comment, "G1 X2 Y0 Z0.8");
    let travel = next_line_index(second, final_segment, "G1 X0 Y0 F7200 ; travel");

    assert_eq!(spiral_segment_count(second), 16);
    assert!(!second.lines().any(|line| line == "G1 Z0.8 F7200 ; lift Z"));
    assert!(comment < final_segment);
    assert!(final_segment < travel);
}

#[test]
fn filament_z_hop_type_overrides_default_layer_change_slope_lift() {
    let output = role_output(json!({
        "filament_z_hop_types": ["Normal Lift", "Slope Lift"]
    }))
    .unwrap();
    let second = layer_section(&output, 1);

    let travel = line_index(second, "G1 X0 Y0 F7200 ; travel");
    let lift = previous_line_index(second, travel, "G1 Z0.8 F7200 ; lift Z");

    assert!(lift < travel);
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 X1.6 Y0 Z0.8 F7200 ; travel")
    );
}

fn next_line_index(section: &str, start: usize, expected: &str) -> usize {
    section
        .lines()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (line == expected).then_some(index))
        .unwrap_or_else(|| panic!("{expected} missing after line {start} from:\n{section}"))
}

fn previous_line_index(section: &str, start: usize, expected: &str) -> usize {
    let lines = section.lines().collect::<Vec<_>>();
    (0..start)
        .rev()
        .find(|index| lines[*index] == expected)
        .unwrap_or_else(|| panic!("{expected} missing before line {start} from:\n{section}"))
}

fn spiral_segment_count(section: &str) -> usize {
    section
        .lines()
        .filter(|line| line.starts_with("G1 X") && line.contains(" Z") && !line.contains(" F"))
        .count()
}
