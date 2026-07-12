use super::*;

#[tokio::test]
async fn filament_minimum_travel_overrides_high_unprefixed_threshold() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 100,
        "filament_retraction_minimum_travel": [0.25, 100],
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E0.8 F1800 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
}

#[tokio::test]
async fn filament_minimum_travel_can_suppress_travel_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "filament_retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
}

#[tokio::test]
async fn invalid_filament_minimum_travel_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(-0.1),
        json!("inf"),
        json!("bad"),
        json!([0.25, "bad"]),
        json!([0.25, -0.1]),
    ] {
        let err = output_result(json!({
            "filament_retraction_minimum_travel": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("filament_retraction_minimum_travel"),
            "filament_retraction_minimum_travel was missing from {err}"
        );
    }
}
