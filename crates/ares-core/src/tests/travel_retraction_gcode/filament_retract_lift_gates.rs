use super::*;

#[tokio::test]
async fn filament_retract_lift_above_overrides_suppressing_unprefixed_lower_gate() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.4,
        "retract_lift_above": 0.3,
        "filament_retract_lift_above": [0.0, 0.3],
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let lift = previous_line_index(&output, travel, "G1 Z0.6 F7200 ; lift Z");
    let restore = next_line_index(&output, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(&output, restore, "G1 E0.8 F1800 ; unretract");

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[tokio::test]
async fn filament_retract_lift_below_zero_overrides_suppressing_unprefixed_upper_gate() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.4,
        "retract_lift_below": 0.1,
        "filament_retract_lift_below": "0,0.1",
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let lift = previous_line_index(&output, travel, "G1 Z0.6 F7200 ; lift Z");
    let restore = next_line_index(&output, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(&output, restore, "G1 E0.8 F1800 ; unretract");

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
}

#[tokio::test]
async fn nil_filament_retract_lift_above_falls_back_to_unprefixed_lower_gate() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.4,
        "retract_lift_above": 100.0,
        "filament_retract_lift_above": "nil,0",
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
