use super::*;

#[tokio::test]
async fn second_layer_transition_emits_changed_nozzle_before_bed_before_z_travel() {
    let output = slice_temperature_output(json!({
        "nozzle_temperature_initial_layer": 205,
        "nozzle_temperature": 215,
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": 55,
        "hot_plate_temp": 60
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";LAYER:1", "M104 S215 ; set nozzle temperature");
    assert_line_after(&output, ";LAYER:1", "M140 S60 ; set bed temperature");
    assert_line_before(
        &output,
        "M104 S215 ; set nozzle temperature",
        "M140 S60 ; set bed temperature",
    );
    assert_line_before(&output, "M140 S60 ; set bed temperature", "G1 Z0.4 F");
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "M104 S215 ; set nozzle temperature")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "M140 S60 ; set bed temperature")
            .count(),
        1
    );
}

#[tokio::test]
async fn zero_or_equal_nozzle_temperature_suppresses_second_layer_nozzle_command() {
    for extra in [
        json!({
            "nozzle_temperature_initial_layer": 205,
            "nozzle_temperature": 0
        }),
        json!({
            "nozzle_temperature_initial_layer": 205,
            "nozzle_temperature": 205
        }),
    ] {
        let output = slice_temperature_output(extra).await.unwrap();

        assert!(
            !output
                .lines()
                .any(|line| line == "M104 S0 ; set nozzle temperature"),
            "{output}"
        );
        assert_eq!(
            output
                .lines()
                .filter(|line| *line == "M104 S205 ; set nozzle temperature")
                .count(),
            1,
            "{output}"
        );
    }
}

#[tokio::test]
async fn second_layer_bed_temperature_can_transition_to_zero() {
    let output = slice_temperature_output(json!({
        "curr_bed_type": "Textured Cool Plate",
        "textured_cool_plate_temp_initial_layer": 45,
        "textured_cool_plate_temp": 0
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";LAYER:1", "M140 S0 ; set bed temperature");
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "M140 S0 ; set bed temperature")
            .count(),
        1
    );
}

#[tokio::test]
async fn default_bed_temperature_formula_uses_highest_second_layer_temperature() {
    let output = slice_temperature_output(json!({
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": [55, 65, 60],
        "hot_plate_temp": [60, 72, 68]
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";LAYER:1", "M140 S72 ; set bed temperature");
}

#[tokio::test]
async fn missing_other_layer_bed_temperature_does_not_emit_redundant_default_transition() {
    for extra in [
        json!({}),
        json!({
            "curr_bed_type": "High Temp Plate",
            "hot_plate_temp_initial_layer": 67
        }),
    ] {
        let output = slice_temperature_output(extra).await.unwrap();

        assert!(
            !output.lines().any(|line| line.starts_with("M140 S")),
            "{output}"
        );
    }
}

#[tokio::test]
async fn missing_other_layer_bed_temperature_reuses_formula_resolved_first_layer_temperature() {
    let output = slice_temperature_output(json!({
        "cool_plate_temp_initial_layer": [35, 65]
    }))
    .await
    .unwrap();

    assert!(
        !output.lines().any(|line| line.starts_with("M140 S")),
        "{output}"
    );
}

#[tokio::test]
async fn klipper_skips_second_layer_temperature_transition() {
    let output = slice_temperature_output(json!({
        "gcode_flavor": "klipper",
        "nozzle_temperature_initial_layer": 205,
        "nozzle_temperature": 215,
        "cool_plate_temp_initial_layer": 35,
        "cool_plate_temp": 40
    }))
    .await
    .unwrap();

    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("M104 S") || line.starts_with("G10 S"))
    );
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("M140 S") || line.starts_with("M190 S"))
    );
}

#[tokio::test]
async fn invalid_other_layer_temperature_values_reach_slice_error() {
    for (key, extra) in [
        ("nozzle_temperature", json!({ "nozzle_temperature": -1 })),
        (
            "hot_plate_temp",
            json!({
                "curr_bed_type": "High Temp Plate",
                "hot_plate_temp": -1
            }),
        ),
    ] {
        let err = slice_temperature_output(extra).await.unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)), "{key}");
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
}

#[tokio::test]
async fn invalid_bed_temperature_formula_reaches_slice_error() {
    let err = slice_temperature_output(json!({
        "bed_temperature_formula": "unsupported"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("bed_temperature_formula"), "{err}");
}

async fn slice_temperature_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0
        }),
        extra,
    );
    slice(square_pyramid_ascii_stl(), options)
        .await
        .map(|output| String::from_utf8(output).unwrap())
}

fn assert_line_before(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| *line == first)
        .unwrap_or_else(|| panic!("missing {first}\n{output}"));
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second))
        .unwrap_or_else(|| panic!("missing {second}\n{output}"));
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn assert_line_after(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| *line == first)
        .unwrap_or_else(|| panic!("missing {first}\n{output}"));
    let second_index = lines
        .iter()
        .position(|line| *line == second)
        .unwrap_or_else(|| panic!("missing {second}\n{output}"));
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
