use super::*;

#[tokio::test]
async fn filament_restart_extra_overrides_layer_change_unretract_distance() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.1,
        "filament_retract_restart_extra": "0.35,9.0",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.5 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.85 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E0.6 F1800 ; unretract")
    );
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E-0.85 F1800 ; retract")
    );
}

#[tokio::test]
async fn zero_filament_restart_extra_overrides_unprefixed_layer_change_restart_extra() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 100,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.3,
        "filament_retract_restart_extra": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.5 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.5 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
    assert!(
        !second
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
}
