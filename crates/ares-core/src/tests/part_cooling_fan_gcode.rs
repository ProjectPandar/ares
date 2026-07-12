use super::*;

#[tokio::test]
async fn default_part_cooling_fan_closes_first_layer_then_emits_full_speed() {
    let output = slice_fan_output(json!({})).await.unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S255"]);
    assert_fan_after_z_before_segment_count(&output, 1, "M106 S255");
    assert_line_after_prefix(&output, "M106 S255", ";LAYER:1");
}

#[tokio::test]
async fn part_cooling_fan_ramp_emits_layer_start_commands_after_z_travel() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 25,
        "fan_max_speed": 75,
        "full_fan_speed_layer": 3,
        "close_fan_the_first_x_layers": 0,
        "slow_down_layer_time": 999,
        "fan_cooling_layer_time": 1000
    }))
    .await
    .unwrap();

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S63", "M106 S127", "M106 S191"]
    );
    assert_fan_after_z_before_segment_count(&output, 0, "M106 S63");
    assert_fan_after_z_before_segment_count(&output, 1, "M106 S127");
    assert_fan_after_z_before_segment_count(&output, 2, "M106 S191");
}

#[tokio::test]
async fn full_fan_speed_layer_one_uses_max_speed_from_first_layer() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 25,
        "fan_max_speed": 75,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S191"]);
    assert_fan_after_z_before_segment_count(&output, 0, "M106 S191");
}

#[tokio::test]
async fn zero_part_cooling_fan_max_speed_suppresses_layer_commands() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 0,
        "fan_max_speed": 0,
        "full_fan_speed_layer": 3
    }))
    .await
    .unwrap();

    assert!(fan_lines(&output).is_empty());
}

#[tokio::test]
async fn invalid_full_fan_speed_layer_reaches_slice_error() {
    let err = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "full_fan_speed_layer": "2.5"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("full_fan_speed_layer"));
}

#[tokio::test]
async fn invalid_part_cooling_fan_speed_reaches_slice_error() {
    let err = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 101
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("fan_max_speed"));
}

#[tokio::test]
async fn part_cooling_fan_min_pwm_clamps_first_layer_command() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 60,
        "full_fan_speed_layer": 3,
        "close_fan_the_first_x_layers": 0,
        "slow_down_layer_time": 999,
        "fan_cooling_layer_time": 1000,
        "part_cooling_fan_min_pwm": 30
    }))
    .await
    .unwrap();

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S76", "M106 S102", "M106 S153"]
    );
    assert_fan_after_z_before_segment_count(&output, 0, "M106 S76");
}

#[tokio::test]
async fn part_cooling_fan_min_pwm_does_not_create_commands_when_fan_disabled() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 0,
        "full_fan_speed_layer": 3,
        "part_cooling_fan_min_pwm": 30
    }))
    .await
    .unwrap();

    assert!(fan_lines(&output).is_empty());
}

#[tokio::test]
async fn invalid_part_cooling_fan_min_pwm_reaches_slice_error() {
    let err = slice_fan_output(json!({ "part_cooling_fan_min_pwm": 30.5 }))
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("part_cooling_fan_min_pwm"));
}

#[tokio::test]
async fn close_fan_zero_preserves_first_layer_fan_command() {
    let output = slice_fan_output(json!({
        "close_fan_the_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S255"]);
    assert_fan_after_z_before_segment_count(&output, 0, "M106 S255");
}

#[tokio::test]
async fn close_fan_two_suppresses_first_two_layers() {
    let output = slice_fan_output(json!({
        "close_fan_the_first_x_layers": 2
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S255"]);
    assert_fan_after_z_before_segment_count(&output, 2, "M106 S255");
}

#[tokio::test]
async fn close_fan_threshold_shifts_part_cooling_ramp() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 60,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 1,
        "slow_down_layer_time": 999,
        "fan_cooling_layer_time": 1000
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S51", "M106 S102"]);
    assert_fan_after_z_before_segment_count(&output, 1, "M106 S51");
    assert_fan_after_z_before_segment_count(&output, 2, "M106 S102");
}

#[tokio::test]
async fn close_fan_threshold_does_not_emit_min_pwm_floor_before_threshold() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 60,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 1,
        "slow_down_layer_time": 999,
        "fan_cooling_layer_time": 1000,
        "part_cooling_fan_min_pwm": 30
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S76", "M106 S102"]);
    assert_fan_after_z_before_segment_count(&output, 1, "M106 S76");
}

#[tokio::test]
async fn long_layer_default_does_not_emit_redundant_fan_off() {
    let output = slice_fan_output(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_cooling_layer_time": 0.0
    }))
    .await
    .unwrap();

    assert!(fan_lines(&output).is_empty());
}

#[tokio::test]
async fn reduce_fan_stop_start_frequency_keeps_long_layer_minimum_fan() {
    let output = slice_fan_output(json!({
        "reduce_fan_stop_start_freq": true,
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_cooling_layer_time": 0.0
    }))
    .await
    .unwrap();

    assert_eq!(fan_lines(&output), vec!["M106 S51"]);
    assert_fan_after_z_before_segment_count(&output, 0, "M106 S51");
}

#[tokio::test]
async fn invalid_close_fan_first_layers_reaches_slice_error() {
    let err = slice_fan_output(json!({ "close_fan_the_first_x_layers": "1.5" }))
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("close_fan_the_first_x_layers"));
}

async fn slice_fan_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        extra,
    );
    slice(tall_pyramid_ascii_stl(), options)
        .await
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 ") || *line == "M126" || *line == "M127")
        .collect()
}

fn assert_fan_after_z_before_segment_count(output: &str, layer: usize, fan_line: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let layer_index = lines
        .iter()
        .position(|line| *line == format!(";LAYER:{layer}"))
        .unwrap();
    let z_index = lines[layer_index..]
        .iter()
        .position(|line| line.starts_with("G1 Z"))
        .map(|index| layer_index + index)
        .unwrap();
    let fan_index = lines[z_index..]
        .iter()
        .position(|line| *line == fan_line)
        .map(|index| z_index + index)
        .unwrap();
    let segment_count_index = lines[z_index..]
        .iter()
        .position(|line| line.starts_with("; segment_count = "))
        .map(|index| z_index + index)
        .unwrap();

    assert!(z_index < fan_index, "{z_index} !< {fan_index}");
    assert!(
        fan_index < segment_count_index,
        "{fan_index} !< {segment_count_index}"
    );
}

fn assert_line_after_prefix(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();

    assert!(
        second_index < first_index,
        "{second_index} !< {first_index}"
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

fn tall_pyramid_ascii_stl() -> Vec<u8> {
    [
        "solid pyramid",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 0.6",
        "vertex 0 1 0.6",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 -1 0.6",
        "vertex 1 0 0.6",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex -1 0 0.6",
        "vertex 0 -1 0.6",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 1 0.6",
        "vertex -1 0 0.6",
        "endloop",
        "endfacet",
        "endsolid pyramid",
    ]
    .join("\n")
    .into_bytes()
}
