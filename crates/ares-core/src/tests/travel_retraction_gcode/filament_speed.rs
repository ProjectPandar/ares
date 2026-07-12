use super::*;

#[tokio::test]
async fn filament_retraction_speed_overrides_travel_retraction_feedrates() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "retraction_speed": 10,
        "deretraction_speed": 20,
        "filament_retraction_speed": [45, 99],
        "filament_deretraction_speed": "55,99",
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F2700 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E0.8 F3300 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
    assert!(!output.lines().any(|line| line == "G1 E-0.8 F600 ; retract"));
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.8 F1200 ; unretract")
    );
}

#[tokio::test]
async fn filament_retraction_length_overrides_travel_retraction_distance() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "retraction_length": 0.5,
        "filament_retraction_length": [1.25, 9.0],
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-1.25 F1800 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E1.25 F1800 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.5 F1800 ; retract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.5 F1800 ; unretract")
    );
}

#[tokio::test]
async fn zero_filament_retraction_length_disables_travel_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "retraction_length": 0.5,
        "filament_retraction_length": 0,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(!output.lines().any(|line| line.ends_with(" ; retract")));
    assert!(!output.lines().any(|line| line.ends_with(" ; unretract")));
}
