use super::*;

#[tokio::test]
async fn filament_z_hop_overrides_layer_change_lift_height() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "z_hop": 0.1,
        "z_hop_types": ["Normal Lift"],
        "filament_z_hop": "0.7,1.0",
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let lift = line_index(second, "G1 Z1.1 F7200 ; lift Z");
    let restore = line_index(second, "G1 Z0.4 F7200 ; restore layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < lift);
    assert!(lift < restore);
    assert!(restore < unretract);
    assert!(!second.lines().any(|line| line == "G1 Z0.5 F7200 ; lift Z"));
}

#[tokio::test]
async fn invalid_filament_z_hop_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(-0.1),
        json!("NaN"),
        json!("bad"),
        json!([0.7, "bad"]),
        json!([0.7, -0.1]),
    ] {
        let err = output_result(json!({
            "filament_z_hop": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("filament_z_hop"),
            "filament_z_hop missing from {err}"
        );
    }
}
