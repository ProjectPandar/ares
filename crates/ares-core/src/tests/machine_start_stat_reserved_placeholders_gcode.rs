use super::*;

#[tokio::test]
async fn machine_start_print_time_sec_renders_numeric_value() {
    let output = slice_machine_start_stat_placeholder_output(json!({
        "machine_start_gcode": ";TIME [print_time_sec]"
    }))
    .await
    .unwrap();
    let value = stat_value_before(&output, ";TIME ", ";LAYER_CHANGE");

    assert_two_decimal_positive_number(value);
    assert!(!output.contains("@PRINT_TIME_SEC@"));
}

#[tokio::test]
async fn machine_start_used_filament_length_renders_numeric_meters() {
    let output = slice_machine_start_stat_placeholder_output(json!({
        "machine_start_gcode": ";FILAMENT [used_filament_length]"
    }))
    .await
    .unwrap();
    let value = stat_value_before(&output, ";FILAMENT ", ";LAYER_CHANGE");

    assert_two_decimal_non_negative_number(value);
    assert!(!output.contains("@USED_FILAMENT_LENGTH@"));
}

#[tokio::test]
async fn machine_start_stat_placeholders_compose_with_existing_placeholders() {
    let output = slice_machine_start_stat_placeholder_output(json!({
        "machine_start_gcode": ";START [print_time_sec] [used_filament_length] [total_layer_count] [num_extruders]",
        "nozzle_diameter": [0.4, 0.6]
    }))
    .await
    .unwrap();
    let line = line_before(&output, ";START ", ";LAYER_CHANGE");
    let values = line
        .strip_prefix(";START ")
        .unwrap()
        .split(' ')
        .collect::<Vec<_>>();

    assert_eq!(values.len(), 4);
    assert_two_decimal_positive_number(values[0]);
    assert_two_decimal_non_negative_number(values[1]);
    assert_eq!(values[2], "2");
    assert_eq!(values[3], "2");
    assert!(!output.contains("@PRINT_TIME_SEC@"));
    assert!(!output.contains("@USED_FILAMENT_LENGTH@"));
}

#[tokio::test]
async fn repeated_machine_start_stat_placeholders_all_render_numeric_values() {
    let output = slice_machine_start_stat_placeholder_output(json!({
        "machine_start_gcode": ";REPEAT [print_time_sec] [print_time_sec] [used_filament_length] [used_filament_length]"
    }))
    .await
    .unwrap();
    let line = line_before(&output, ";REPEAT ", ";LAYER_CHANGE");
    let values = line
        .strip_prefix(";REPEAT ")
        .unwrap()
        .split(' ')
        .collect::<Vec<_>>();

    assert_eq!(values.len(), 4);
    assert_two_decimal_positive_number(values[0]);
    assert_eq!(values[0], values[1]);
    assert_two_decimal_non_negative_number(values[2]);
    assert_eq!(values[2], values[3]);
    assert!(!output.contains("@PRINT_TIME_SEC@"));
    assert!(!output.contains("@USED_FILAMENT_LENGTH@"));
}

#[tokio::test]
async fn stat_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_machine_start_stat_placeholder_output(json!({
        "layer_change_gcode": ";LC [print_time_sec] [used_filament_length] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [print_time_sec] [used_filament_length] 1",
        "; segment_count = 4",
    );
}

async fn slice_machine_start_stat_placeholder_output(
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

fn assert_line_before(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();

    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn line_before<'a>(output: &'a str, first_prefix: &str, second: &str) -> &'a str {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| line.starts_with(first_prefix))
        .unwrap();
    let second_index = lines.iter().position(|line| *line == second).unwrap();

    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
    lines[first_index]
}

fn stat_value_before<'a>(output: &'a str, first_prefix: &str, second: &str) -> &'a str {
    line_before(output, first_prefix, second)
        .strip_prefix(first_prefix)
        .unwrap()
}

fn assert_two_decimal_positive_number(value: &str) {
    assert_two_decimal_number(value);
    assert!(value.parse::<f64>().unwrap() > 0.0);
}

fn assert_two_decimal_non_negative_number(value: &str) {
    assert_two_decimal_number(value);
    assert!(value.parse::<f64>().unwrap() >= 0.0);
}

fn assert_two_decimal_number(value: &str) {
    let (whole, fractional) = value.split_once('.').unwrap();
    assert!(!whole.is_empty());
    assert_eq!(fractional.len(), 2);
    value.parse::<f64>().unwrap();
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
