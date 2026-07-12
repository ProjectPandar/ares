use super::*;

#[tokio::test]
async fn firmware_retraction_emits_g10_g11_around_second_layer_z() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "use_firmware_retraction": true,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G10 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G11 ; unretract");
    let first_print = first_extrusion_line_index(second);

    assert!(retract < z);
    assert!(z < unretract);
    assert!(unretract < first_print);
    assert!(!second.lines().any(|line| line.starts_with("G1 E-")));
    assert!(
        !second
            .lines()
            .any(|line| line.starts_with("G1 E") && line.contains("unretract"))
    );
}

#[tokio::test]
async fn firmware_retraction_keeps_disabled_and_zero_length_layer_change_gates() {
    let disabled = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 100,
        "use_firmware_retraction": true,
        "z_hop": 0
    }))
    .await;
    let zero_length = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0,
        "use_firmware_retraction": true,
        "z_hop": 0
    }))
    .await;

    for output in [disabled, zero_length] {
        assert!(!output.lines().any(|line| line == "G10 ; retract"));
        assert!(!output.lines().any(|line| line == "G11 ; unretract"));
        assert!(!output.lines().any(is_layer_retraction_line));
    }
}

#[tokio::test]
async fn firmware_retraction_rejects_non_boolean_runtime_values() {
    let err = output_result(json!({
        "retract_when_changing_layer": true,
        "use_firmware_retraction": "true",
        "z_hop": 0
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("use_firmware_retraction"));
}

#[tokio::test]
async fn firmware_retraction_preserves_absolute_e_print_state() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "use_firmware_retraction": true,
        "use_relative_e_distances": false,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);
    let unretract = line_index(second, "G11 ; unretract");
    let first_print_index = first_extrusion_line_index(second);
    let first_print_e = e_value(second.lines().nth(first_print_index).unwrap());

    assert!(unretract < first_print_index);
    assert!(first_print_e > 0.0);
}
