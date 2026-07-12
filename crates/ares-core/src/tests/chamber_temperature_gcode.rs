use super::*;

#[tokio::test]
async fn default_slice_does_not_emit_chamber_temperature_commands() {
    let output = slice_chamber_output(json!({})).await.unwrap();

    assert_no_chamber_commands(&output);
}

#[tokio::test]
async fn enabled_chamber_temperature_emits_startup_and_shutdown_commands() {
    let output = slice_chamber_output(json!({
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        "M191 S45 ;set chamber_temperature and wait for it to be reached",
        ";LAYER_CHANGE",
    );
    assert_line_before(&output, "M141 S0;set chamber_temperature", "M2");
}

#[tokio::test]
async fn chamber_temperature_uses_any_activation_and_max_temperature() {
    let output = slice_chamber_output(json!({
        "activate_chamber_temp_control": [false, true],
        "chamber_temperature": [40, 55, 45]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        "M191 S55 ;set chamber_temperature and wait for it to be reached",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn zero_chamber_temperature_suppresses_commands_even_when_enabled() {
    let output = slice_chamber_output(json!({
        "activate_chamber_temp_control": true,
        "chamber_temperature": 0
    }))
    .await
    .unwrap();

    assert_no_chamber_commands(&output);
}

#[tokio::test]
async fn klipper_skips_chamber_temperature_commands() {
    let output = slice_chamber_output(json!({
        "gcode_flavor": "klipper",
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .await
    .unwrap();

    assert_no_chamber_commands(&output);
}

#[tokio::test]
async fn invalid_chamber_temperature_value_reaches_slice_error() {
    let err = slice_chamber_output(json!({
        "activate_chamber_temp_control": true,
        "chamber_temperature": -1
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("chamber_temperature"));
}

#[tokio::test]
async fn invalid_chamber_activation_value_reaches_slice_error() {
    let err = slice_chamber_output(json!({
        "activate_chamber_temp_control": "True",
        "chamber_temperature": 45
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("activate_chamber_temp_control"));
}

#[tokio::test]
async fn machine_start_gcode_chamber_command_suppresses_automatic_chamber_startup_command() {
    let output = slice_chamber_output(json!({
        "machine_start_gcode": "M191 S45",
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .await
    .unwrap();

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("M191 S"))
            .collect::<Vec<_>>(),
        vec!["M191 S45"]
    );
    assert_line_before(&output, "M141 S0;set chamber_temperature", "M2");
}

#[tokio::test]
async fn machine_start_gcode_chamber_non_wait_command_suppresses_automatic_chamber_startup_command()
{
    let output = slice_chamber_output(json!({
        "machine_start_gcode": "M141 S45",
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .await
    .unwrap();

    assert!(
        !output.lines().any(|line| {
            line == "M191 S45 ;set chamber_temperature and wait for it to be reached"
        })
    );
    assert!(output.lines().any(|line| line == "M141 S45"));
    assert_line_before(&output, "M141 S0;set chamber_temperature", "M2");
}

async fn slice_chamber_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn assert_no_chamber_commands(output: &str) {
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("M191 S") || line.starts_with("M141 S"))
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
