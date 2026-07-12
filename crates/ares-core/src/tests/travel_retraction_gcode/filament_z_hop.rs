use super::*;

#[tokio::test]
async fn filament_z_hop_overrides_travel_retraction_lift_height() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.1,
        "filament_z_hop": [0.7, 1.0],
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let lift = previous_line_index(&output, travel, "G1 Z0.9 F7200 ; lift Z");
    let restore = next_line_index(&output, travel, "G1 Z0.2 F7200 ; restore layer Z");
    let unretract = next_line_index(&output, restore, "G1 E0.8 F1800 ; unretract");

    assert!(retract < lift);
    assert!(lift < travel);
    assert!(travel < restore);
    assert!(restore < unretract);
    assert!(!output.lines().any(|line| line == "G1 Z0.3 F7200 ; lift Z"));
}

#[tokio::test]
async fn zero_filament_z_hop_disables_travel_lift_without_disabling_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop_types": ["Normal Lift"],
        "z_hop": 0.7,
        "filament_z_hop": 0,
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
