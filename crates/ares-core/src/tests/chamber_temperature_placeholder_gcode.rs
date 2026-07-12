use super::*;

#[tokio::test]
async fn machine_start_chamber_temperature_renders_configured_vector_and_overall_max() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "machine_start_gcode": ";CHAMBER [chamber_temperature] [overall_chamber_temperature]",
        "chamber_temperature": [40, 55, 45]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CHAMBER 40,55,45 55", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_chamber_temperature_defaults_to_zero() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "machine_start_gcode": ";CHAMBER [chamber_temperature] [overall_chamber_temperature]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CHAMBER 0 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_chamber_temperature_accepts_numeric_string_and_composes() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "machine_start_gcode": ";START [chamber_temperature] [overall_chamber_temperature] [total_layer_count]",
        "chamber_temperature": "41;55"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 41,55 55 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_chamber_temperature_ignores_activation_for_placeholder_rendering() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "machine_start_gcode": ";CHAMBER [overall_chamber_temperature]",
        "activate_chamber_temp_control": false,
        "chamber_temperature": [35, 50]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CHAMBER 50", ";LAYER_CHANGE");
    assert_no_chamber_startup_or_shutdown_commands(&output);
}

#[tokio::test]
async fn machine_start_overall_chamber_temperature_suppresses_auto_chamber_startup() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "machine_start_gcode": "M191 S[overall_chamber_temperature]",
        "activate_chamber_temp_control": true,
        "chamber_temperature": [40, 55]
    }))
    .await
    .unwrap();

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("M191 S"))
            .collect::<Vec<_>>(),
        vec!["M191 S55"]
    );
    assert_line_before(&output, "M141 S0;set chamber_temperature", "M2");
}

#[tokio::test]
async fn machine_start_chamber_temperature_stays_literal_in_layer_change() {
    let output = slice_chamber_temperature_placeholder_output(json!({
        "layer_change_gcode": ";LC [chamber_temperature] [overall_chamber_temperature] [layer_num]",
        "chamber_temperature": [41, 55]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [chamber_temperature] [overall_chamber_temperature] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_chamber_temperature_rejects_invalid_values() {
    for extra in [
        json!({
            "machine_start_gcode": ";CHAMBER [chamber_temperature]",
            "chamber_temperature": -1
        }),
        json!({
            "machine_start_gcode": ";CHAMBER [overall_chamber_temperature]",
            "chamber_temperature": []
        }),
        json!({
            "machine_start_gcode": ";CHAMBER [chamber_temperature]",
            "chamber_temperature": [35, "bad"]
        }),
    ] {
        let err = slice_chamber_temperature_placeholder_output(extra)
            .await
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("chamber_temperature"));
    }
}

async fn slice_chamber_temperature_placeholder_output(
    extra: serde_json::Value,
) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        extra,
    );
    slice(square_pyramid_ascii_stl(), options)
        .await
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn assert_no_chamber_startup_or_shutdown_commands(output: &str) {
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("M191 S") || line == "M141 S0;set chamber_temperature")
    );
}

fn assert_line_before(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();

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
