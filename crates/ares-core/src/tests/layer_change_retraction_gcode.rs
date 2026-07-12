use super::*;

use crate::PrintPathRole;

mod filament_restart_extra;
mod filament_retract_lift_gates;
mod filament_retract_when_changing_layer;
mod filament_speed;
mod filament_z_hop;
mod firmware;
mod z_hop;
mod z_hop_type;

#[tokio::test]
async fn default_preserves_existing_output_bytes() {
    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(output.len(), 4753);
    assert_eq!(fnv1a64(&output), 0x8990a54281eb9dfd);
}

#[tokio::test]
async fn false_emits_no_retraction_and_keeps_known_movement_lines() {
    let disabled = output_for(json!({
        "retract_when_changing_layer": [false],
        "retraction_minimum_travel": 100,
        "seam_gap": 0
    }))
    .await;

    assert!(!disabled.lines().any(is_layer_retraction_line));
    assert!(disabled.lines().any(|line| line == "G1 Z0.2 F7200"));
    assert!(disabled.lines().any(|line| line == "G1 Z0.4 F7200"));
    assert!(disabled.lines().any(|line| line == "G1 X-0.5 Y0 E0.02393"));
}

#[tokio::test]
async fn enabled_emits_relative_retract_before_second_layer_z_and_unretract_before_print() {
    let output = output_for(json!({
        "retract_when_changing_layer": [true],
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        !layer_section(&output, 0)
            .lines()
            .any(is_layer_retraction_line)
    );
    let second = layer_section(&output, 1);
    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(retract < z);
    assert!(z < unretract);
    assert!(unretract < first_print);
}

#[tokio::test]
async fn custom_length_and_speeds_select_first_vector_values() {
    let output = output_for(json!({
        "retract_when_changing_layer": [true, false],
        "retraction_length": [1.25, 9.0],
        "retraction_speed": "25,99",
        "deretraction_speed": "40;99",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-1.25 F1500 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E1.25 F2400 ; unretract")
    );
}

#[tokio::test]
async fn restart_extra_adds_to_unretract_without_changing_retract() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.12,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-0.5 F1800 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E0.62 F1800 ; unretract")
    );
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E-0.62 F1800 ; retract")
    );
}

#[tokio::test]
async fn restart_extra_selects_first_array_value_and_keeps_deretraction_speed() {
    let output = output_for(json!({
        "retract_when_changing_layer": [true, false],
        "retraction_length": [1.0, 9.0],
        "retract_restart_extra": [0.2, 9.0],
        "retraction_speed": 25,
        "deretraction_speed": 40,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(second.lines().any(|line| line == "G1 E-1 F1500 ; retract"));
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E1.2 F2400 ; unretract")
    );
}

#[tokio::test]
async fn restart_extra_selects_first_string_value() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.75,
        "retract_restart_extra": "0.15,9",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-0.75 F1800 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E0.9 F1800 ; unretract")
    );
}

#[tokio::test]
async fn deretraction_speed_zero_uses_retraction_speed() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.5,
        "retraction_speed": 20,
        "deretraction_speed": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-0.5 F1200 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E0.5 F1200 ; unretract")
    );
}

#[tokio::test]
async fn zero_length_disables_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(!output.lines().any(is_layer_retraction_line));
}

#[tokio::test]
async fn absolute_extrusion_mode_updates_e_state_around_layer_change() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.25,
        "use_relative_e_distances": false,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);
    let retract_e = e_value(
        second
            .lines()
            .find(|line| line.ends_with(" ; retract"))
            .unwrap(),
    );
    let unretract_e = e_value(
        second
            .lines()
            .find(|line| line.ends_with(" ; unretract"))
            .unwrap(),
    );

    assert!((unretract_e - retract_e - 0.75).abs() < 0.00001);
    let first_print_e = e_value(
        second
            .lines()
            .nth(first_extrusion_line_index(second))
            .unwrap(),
    );
    assert!(first_print_e > unretract_e);
}

#[tokio::test]
async fn bare_commands_are_emitted_when_comments_are_disabled() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.5,
        "z_hop": 0
    }))
    .await;

    assert!(output.lines().any(|line| line == "G1 E-0.5 F1800"));
    assert!(output.lines().any(|line| line == "G1 E0.5 F1800"));
    assert!(!output.lines().any(|line| line.ends_with(" ; retract")));
    assert!(!output.lines().any(|line| line.ends_with(" ; unretract")));
}

#[tokio::test]
async fn invalid_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("retract_when_changing_layer", json!([])),
        ("retract_when_changing_layer", json!(["true"])),
        ("retract_when_changing_layer", json!([true, "bad"])),
        ("retraction_length", json!([])),
        ("retraction_length", json!(-0.1)),
        ("retract_restart_extra", json!([])),
        ("retract_restart_extra", json!(-0.1)),
        ("retract_restart_extra", json!("inf")),
        ("retraction_speed", json!(-1)),
        ("deretraction_speed", json!("inf")),
    ] {
        let err = slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({
                key: value
            }))
            .unwrap(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains(key),
            "{key} was missing from {err}"
        );
    }
}

pub(super) async fn output_result(extra: serde_json::Value) -> Result<String, SliceError> {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    options
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(options).unwrap(),
    )
    .await?;

    Ok(String::from_utf8(output).unwrap())
}

pub(super) async fn output_for(extra: serde_json::Value) -> String {
    output_result(extra).await.unwrap()
}

pub(super) fn synthetic_role_layers_output(
    extra: serde_json::Value,
    roles_by_layer: Vec<Vec<PrintPathRole>>,
) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        }),
        extra,
    );
    let pipeline =
        crate::pipeline::layer_change_test_support::role_layers_pipeline(&options, roles_by_layer);
    crate::gcode::format_gcode(&pipeline, &options).map(|bytes| String::from_utf8(bytes).unwrap())
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

pub(super) fn layer_section(output: &str, layer: u32) -> &str {
    let marker = format!(";LAYER:{layer}\n");
    let start = output.find(&marker).unwrap();
    let rest = &output[start..];
    let next = rest[marker.len()..]
        .find("\n;LAYER_CHANGE\n")
        .map(|next| marker.len() + next)
        .unwrap_or(rest.len());
    &rest[..next]
}

pub(super) fn line_index(section: &str, expected: &str) -> usize {
    section
        .lines()
        .position(|line| line == expected)
        .unwrap_or_else(|| panic!("{expected} missing from:\n{section}"))
}

pub(super) fn first_extrusion_line_index(section: &str) -> usize {
    section
        .lines()
        .position(|line| line.starts_with("G1 X") && line.contains(" E"))
        .unwrap_or_else(|| panic!("first extrusion line missing from:\n{section}"))
}

pub(super) fn is_layer_retraction_line(line: &str) -> bool {
    line.ends_with(" ; retract")
        || line.ends_with(" ; unretract")
        || line.starts_with("G1 E-")
        || (line.starts_with("G1 E") && line.contains(" ; unretract"))
}

pub(super) fn e_value(line: &str) -> f64 {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix('E'))
        .unwrap()
        .parse()
        .unwrap()
}
