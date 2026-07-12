use super::*;

#[tokio::test]
async fn filament_restart_extra_overrides_travel_unretract_distance() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.1,
        "filament_retract_restart_extra": [0.35, 9.0],
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.5 F1800 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E0.85 F1800 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.6 F1800 ; unretract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.85 F1800 ; retract")
    );
}

#[tokio::test]
async fn zero_filament_restart_extra_overrides_unprefixed_travel_restart_extra() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.3,
        "filament_retract_restart_extra": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.5 F1800 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E0.5 F1800 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
}

#[tokio::test]
async fn invalid_filament_restart_extra_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(-0.1),
        json!("inf"),
        json!("bad"),
        json!([0.25, "bad"]),
        json!([0.25, -0.1]),
        json!([0.25, "inf"]),
    ] {
        let err = output_result(json!({
            "filament_retract_restart_extra": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("filament_retract_restart_extra"),
            "filament_retract_restart_extra was missing from {err}"
        );
    }
}
