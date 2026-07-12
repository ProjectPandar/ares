use super::*;

#[tokio::test]
async fn filament_retraction_speed_overrides_layer_change_feedrates() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "retraction_speed": 10,
        "deretraction_speed": 20,
        "filament_retraction_speed": [45, 99],
        "filament_deretraction_speed": "55,99",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F2700 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F3300 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
    assert!(!second.lines().any(|line| line == "G1 E-0.8 F600 ; retract"));
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E0.8 F1200 ; unretract")
    );
}

#[tokio::test]
async fn filament_deretraction_speed_zero_uses_effective_filament_retraction_speed() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "retraction_speed": 10,
        "deretraction_speed": 20,
        "filament_retraction_speed": 45,
        "filament_deretraction_speed": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 E-0.8 F2700 ; retract")
    );
    assert!(
        second
            .lines()
            .any(|line| line == "G1 E0.8 F2700 ; unretract")
    );
}

#[tokio::test]
async fn invalid_filament_retraction_speed_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("filament_retraction_speed", json!([])),
        ("filament_retraction_speed", json!(-1)),
        ("filament_retraction_speed", json!("NaN")),
        ("filament_deretraction_speed", json!([])),
        ("filament_deretraction_speed", json!(-1)),
        ("filament_deretraction_speed", json!("inf")),
    ] {
        let err = output_result(json!({
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key} missing from {err}");
    }
}

#[tokio::test]
async fn filament_retraction_length_overrides_layer_change_distance() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "retraction_length": 0.5,
        "filament_retraction_length": "1.25,9.0",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-1.25 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E1.25 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E-0.5 F1800 ; retract")
    );
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E0.5 F1800 ; unretract")
    );
}

#[tokio::test]
async fn zero_filament_retraction_length_disables_layer_change_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_length": 0.5,
        "filament_retraction_length": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(!output.lines().any(is_layer_retraction_line));
}

#[tokio::test]
async fn invalid_filament_retraction_length_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(-0.1),
        json!("NaN"),
        json!("bad"),
        json!([1.0, "bad"]),
        json!([1.0, -0.1]),
    ] {
        let err = output_result(json!({
            "filament_retraction_length": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("filament_retraction_length"),
            "filament_retraction_length missing from {err}"
        );
    }
}
