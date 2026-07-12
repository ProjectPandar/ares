use super::*;

#[tokio::test]
async fn default_slice_does_not_emit_exhaust_fan_commands() {
    let output = slice_exhaust_output(json!({})).await.unwrap();

    assert!(exhaust_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn enabled_during_print_exhaust_fan_emits_before_first_layer() {
    let output = slice_exhaust_output(json!({
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": false,
        "during_print_exhaust_fan_speed": 60
    }))
    .await
    .unwrap();

    assert_eq!(exhaust_fan_lines(&output), vec!["M106 P3 S153"]);
    assert_line_before(&output, "M106 P3 S153", ";LAYER_CHANGE");
}

#[tokio::test]
async fn enabled_completion_exhaust_fan_emits_before_m2() {
    let output = slice_exhaust_output(json!({
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": false,
        "activate_air_filtration_on_completion": true,
        "complete_print_exhaust_fan_speed": 80
    }))
    .await
    .unwrap();

    assert_eq!(exhaust_fan_lines(&output), vec!["M106 P3 S204"]);
    assert_line_before(&output, "M106 P3 S204", "M2");
}

#[tokio::test]
async fn enabled_exhaust_fan_emits_during_print_then_completion_commands() {
    let output = slice_exhaust_output(json!({
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 60,
        "complete_print_exhaust_fan_speed": 80
    }))
    .await
    .unwrap();

    assert_eq!(
        exhaust_fan_lines(&output),
        vec!["M106 P3 S153", "M106 P3 S204"]
    );
    assert_line_before(&output, "M106 P3 S153", ";LAYER_CHANGE");
    assert_line_before(&output, "M106 P3 S153", "M106 P3 S204");
    assert_line_before(&output, "M106 P3 S204", "M2");
}

#[tokio::test]
async fn klipper_skips_exhaust_fan_commands() {
    let output = slice_exhaust_output(json!({
        "gcode_flavor": "klipper",
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 60,
        "complete_print_exhaust_fan_speed": 80
    }))
    .await
    .unwrap();

    assert!(exhaust_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn invalid_exhaust_fan_speed_reaches_slice_error() {
    let err = slice_exhaust_output(json!({
        "activate_air_filtration": true,
        "during_print_exhaust_fan_speed": 101
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("during_print_exhaust_fan_speed"));
}

async fn slice_exhaust_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn exhaust_fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 P3 S"))
        .collect()
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
