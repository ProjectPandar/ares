use super::*;

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_renders_configured_vector() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
        "during_print_exhaust_fan_speed": [60, 80, 100]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EXHAUST 153,204,255", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_defaults_to_orca_percent() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EXHAUST 153", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_accepts_numeric_scalar() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
        "during_print_exhaust_fan_speed": 80
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EXHAUST 204", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_accepts_numeric_string_and_composes() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";START [during_print_exhaust_fan_speed_num] [total_layer_count]",
        "during_print_exhaust_fan_speed": "20;60"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 51,153 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_ignores_air_filtration_flags() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
        "support_air_filtration": false,
        "activate_air_filtration": false,
        "activate_air_filtration_during_print": false,
        "during_print_exhaust_fan_speed": [80]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EXHAUST 204", ";LAYER_CHANGE");
    assert!(exhaust_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_preserves_auto_exhaust_ordering() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": false,
        "during_print_exhaust_fan_speed": [60]
    }))
    .await
    .unwrap();

    assert_eq!(exhaust_fan_lines(&output), vec!["M106 P3 S153"]);
    assert_line_before(&output, "M106 P3 S153", ";EXHAUST 153");
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_stays_literal_in_layer_change() {
    let output = slice_exhaust_fan_speed_num_placeholder_output(json!({
        "layer_change_gcode": ";LC [during_print_exhaust_fan_speed_num] [layer_num]",
        "during_print_exhaust_fan_speed": [80]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [during_print_exhaust_fan_speed_num] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_during_print_exhaust_fan_speed_num_rejects_invalid_values() {
    for extra in [
        json!({
            "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
            "during_print_exhaust_fan_speed": 101
        }),
        json!({
            "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
            "during_print_exhaust_fan_speed": -1
        }),
        json!({
            "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
            "during_print_exhaust_fan_speed": []
        }),
        json!({
            "machine_start_gcode": ";EXHAUST [during_print_exhaust_fan_speed_num]",
            "during_print_exhaust_fan_speed": [60, "bad"]
        }),
    ] {
        let err = slice_exhaust_fan_speed_num_placeholder_output(extra)
            .await
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("during_print_exhaust_fan_speed"));
    }
}

async fn slice_exhaust_fan_speed_num_placeholder_output(
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
