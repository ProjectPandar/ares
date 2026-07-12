use super::*;

#[tokio::test]
async fn filament_retract_when_changing_layer_enables_layer_change_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "filament_retract_when_changing_layer": [true, false],
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
}

#[tokio::test]
async fn filament_retract_when_changing_layer_disables_unprefixed_layer_change_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "filament_retract_when_changing_layer": false,
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    assert!(
        second
            .lines()
            .any(|line| line == "G1 Z0.4 F7200 ; move to layer Z")
    );
    assert!(!second.lines().any(is_layer_retraction_line));
}

#[tokio::test]
async fn serialized_filament_retract_when_changing_layer_uses_first_value() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "filament_retract_when_changing_layer": "1,0",
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
}

#[tokio::test]
async fn nil_filament_retract_when_changing_layer_falls_back_to_unprefixed_layer_change_retraction()
{
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "filament_retract_when_changing_layer": [null, false],
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);

    let retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let z = line_index(second, "G1 Z0.4 F7200 ; move to layer Z");
    let unretract = line_index(second, "G1 E0.8 F1800 ; unretract");

    assert!(retract < z);
    assert!(z < unretract);
}

#[tokio::test]
async fn invalid_filament_retract_when_changing_layer_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!("true"),
        json!("1,bad"),
        json!([true, "bad"]),
        json!([null, 1]),
    ] {
        let err = output_result(json!({
            "filament_retract_when_changing_layer": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("filament_retract_when_changing_layer"),
            "filament_retract_when_changing_layer missing from {err}"
        );
    }
}
