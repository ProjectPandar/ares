use super::*;

use serde_json::{Value, json};

use crate::PrintPathRole;

fn role_layers_output(
    extra: Value,
    roles_by_layer: Vec<Vec<PrintPathRole>>,
) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "gcode_comments": true
    });
    let options = options_json.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline =
        crate::pipeline::layer_change_test_support::role_layers_pipeline(&options, roles_by_layer);
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

fn role_output(extra: Value, roles: Vec<PrintPathRole>) -> Result<String, SliceError> {
    role_layers_output(extra, vec![roles])
}

fn lift_count(gcode: &str) -> usize {
    gcode.matches("; lift Z").count()
}

#[tokio::test]
async fn default_z_hop_lifts_after_ordinary_travel_retract_and_restores_before_unretract() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let lift = previous_line_index(&output, travel, "G1 Z0.6 F7200 ; lift Z");
    let restore = next_line_index(&output, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(&output, restore, "G1 E0.8 F1800 ; unretract");
    let next_print = next_print_line_index(&output, travel);

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
    assert!(unretract < next_print);
}

#[tokio::test]
async fn firmware_ordinary_travel_retraction_lifts_and_restores_around_g10_g11() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "use_firmware_retraction": true,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G10 ; retract");
    let lift = previous_line_index(&output, travel, "G1 Z0.6 F7200 ; lift Z");
    let restore = next_line_index(&output, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(&output, restore, "G11 ; unretract");
    let next_print = next_print_line_index(&output, travel);

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
    assert!(unretract < next_print);
}

#[tokio::test]
async fn zero_z_hop_preserves_no_hop_ordinary_travel_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
    assert!(!output.lines().any(|line| line.contains("lift Z")));
    assert!(!output.lines().any(|line| line.contains("restore layer Z")));
}

#[tokio::test]
async fn ordinary_travel_z_hop_respects_lift_above_and_below_gates() {
    let below_lower = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.3,
        "retract_lift_above": 0.3,
        "gcode_comments": true
    }))
    .await;
    let above_upper = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.3,
        "retract_lift_below": 0.1,
        "gcode_comments": true
    }))
    .await;
    let inside = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.3,
        "retract_lift_above": 0.2,
        "retract_lift_below": 0.2,
        "gcode_comments": true
    }))
    .await;

    for output in [&below_lower, &above_upper] {
        assert!(
            output
                .lines()
                .any(|line| line == "G1 E-0.8 F1800 ; retract")
        );
        assert!(
            output
                .lines()
                .any(|line| line == "G1 E0.8 F1800 ; unretract")
        );
        assert!(!output.lines().any(|line| line.contains("lift Z")));
        assert!(!output.lines().any(|line| line.contains("restore layer Z")));
    }
    assert!(inside.lines().any(|line| line == "G1 Z0.5 F7200 ; lift Z"));
    assert!(
        inside
            .lines()
            .any(|line| line == "G1 Z0.2 F7200 ; restore layer Z")
    );
}

#[tokio::test]
async fn pending_travel_z_hop_crosses_layer_change_without_restoring_previous_layer_z() {
    let output = boundary_output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "gcode_comments": true
    }));
    let first_retract = line_index(&output, "G1 E-0.8 F1800 ; retract");
    let lift = next_line_index(&output, first_retract, "G1 Z0.6 F7200 ; lift Z");
    let layer_change = next_line_index(&output, lift, ";LAYER_CHANGE");
    let move_to_layer = next_line_index(&output, layer_change, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = next_line_index(&output, move_to_layer, "G1 E0.8 F1800 ; unretract");
    let next_print = next_print_line_index(&output, move_to_layer);

    assert!(
        !output
            .lines()
            .enumerate()
            .skip(layer_change)
            .take(unretract - layer_change)
            .any(|(_, line)| line == "G1 Z0.2 F7200 ; restore layer Z")
    );
    assert!(first_retract < lift);
    assert!(lift < layer_change);
    assert!(move_to_layer < unretract);
    assert!(unretract < next_print);
}

#[test]
fn retract_lift_enforce_top_only_suppresses_non_top_ordinary_travel_lift() {
    let gcode = role_output(
        json!({
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::ExternalPerimeter,
        ],
    )
    .unwrap();
    let layer = layer_section(&gcode, 0);

    assert!(layer.contains("G1 E-0.8"));
    assert!(layer.contains("; travel"));
    assert!(layer.contains("G1 E0"));
    assert!(!layer.contains("; lift Z"));
    assert!(!layer.contains("; restore layer Z"));
}

#[test]
fn retract_lift_enforce_top_only_allows_top_ordinary_travel_lift() {
    let gcode = role_output(
        json!({
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            PrintPathRole::TopSolidInfill,
            PrintPathRole::ExternalPerimeter,
        ],
    )
    .unwrap();
    let layer = layer_section(&gcode, 0);

    assert!(layer.contains("G1 Z0.6 F7200 ; lift Z"));
    assert!(layer.contains("G1 Z0.2 F7200 ; restore layer Z"));
}

#[test]
fn retract_lift_enforce_bottom_only_allows_first_layer_ordinary_travel_lift_only() {
    let first_layer = role_output(
        json!({
            "retract_lift_enforce": "Bottom Only",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::ExternalPerimeter,
        ],
    )
    .unwrap();
    assert!(layer_section(&first_layer, 0).contains("G1 Z0.6 F7200 ; lift Z"));

    let later_layer = role_layers_output(
        json!({
            "retract_lift_enforce": "Bottom Only",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![
                PrintPathRole::ExternalPerimeter,
                PrintPathRole::ExternalPerimeter,
            ],
        ],
    )
    .unwrap();
    let layer = layer_section(&later_layer, 1);

    assert!(layer.contains("G1 E-0.8"));
    assert!(layer.contains("; travel"));
    assert!(layer.contains("G1 E0"));
    assert!(!layer.contains("; lift Z"));
}

#[test]
fn retract_lift_enforce_top_and_bottom_allows_later_top_ordinary_travel_lift() {
    let gcode = role_layers_output(
        json!({
            "retract_lift_enforce": "Top and Bottom",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![
                PrintPathRole::TopSolidInfill,
                PrintPathRole::ExternalPerimeter,
            ],
        ],
    )
    .unwrap();
    let layer = layer_section(&gcode, 1);

    assert!(layer.contains("G1 Z0.8 F7200 ; lift Z"));
    assert!(layer.contains("G1 Z0.4 F7200 ; restore layer Z"));
}

#[test]
fn retract_lift_enforce_gap_fill_preserves_previous_top_for_ordinary_travel() {
    let gcode = role_output(
        json!({
            "retract_lift_enforce": "Top Only",
            "z_hop_types": ["Normal Lift"],
            "z_hop": 0.4,
            "retraction_length": 0.8
        }),
        vec![
            PrintPathRole::TopSolidInfill,
            PrintPathRole::GapFill,
            PrintPathRole::ExternalPerimeter,
        ],
    )
    .unwrap();
    let layer = layer_section(&gcode, 0);

    assert_eq!(lift_count(layer), 2);
}
