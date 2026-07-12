use super::*;

#[tokio::test]
async fn default_slice_does_not_emit_auxiliary_fan_commands() {
    let output = slice_auxiliary_fan_output(json!({})).await.unwrap();

    assert!(auxiliary_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn disabled_auxiliary_fan_ignores_configured_speed() {
    let output = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": false,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert!(auxiliary_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn enabled_auxiliary_fan_zero_speed_emits_no_commands() {
    let output = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 0,
        "close_additional_fan_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert!(auxiliary_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn enabled_auxiliary_fan_emits_after_default_close_threshold_and_closes_before_m2() {
    let output = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70
    }))
    .await
    .unwrap();

    assert_eq!(
        auxiliary_fan_lines(&output),
        vec!["M106 P2 S178", "M106 P2 S0"]
    );
    assert_line_after_prefix(&output, "M106 P2 S178", ";LAYER:1");
    assert_fan_after_z_before_segment_count(&output, 1, "M106 P2 S178");
    assert_line_before(&output, "M106 P2 S0", "M2");
}

#[tokio::test]
async fn close_additional_fan_zero_allows_auxiliary_fan_on_first_layer() {
    let output = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert_eq!(
        auxiliary_fan_lines(&output),
        vec!["M106 P2 S178", "M106 P2 S0"]
    );
    assert_fan_after_z_before_segment_count(&output, 0, "M106 P2 S178");
}

#[tokio::test]
async fn klipper_skips_auxiliary_fan_commands() {
    let output = slice_auxiliary_fan_output(json!({
        "gcode_flavor": "klipper",
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 0
    }))
    .await
    .unwrap();

    assert!(auxiliary_fan_lines(&output).is_empty());
}

#[tokio::test]
async fn auxiliary_fan_ramps_until_full_speed_layer() {
    let output = slice_auxiliary_fan_output(json!({
        "layer_height": 0.08,
        "initial_layer_print_height": 0.08,
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 80,
        "close_additional_fan_first_x_layers": 2,
        "additional_fan_full_speed_layer": 5
    }))
    .await
    .unwrap();

    assert_eq!(
        auxiliary_fan_lines(&output),
        vec!["M106 P2 S68", "M106 P2 S135", "M106 P2 S204", "M106 P2 S0"]
    );
    assert_fan_after_z_before_segment_count(&output, 2, "M106 P2 S68");
    assert_fan_after_z_before_segment_count(&output, 3, "M106 P2 S135");
    assert_fan_after_z_before_segment_count(&output, 4, "M106 P2 S204");
}

#[tokio::test]
async fn auxiliary_fan_ignores_part_cooling_close_layers() {
    let output = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_fan_the_first_x_layers": 3
    }))
    .await
    .unwrap();

    assert_eq!(
        auxiliary_fan_lines(&output),
        vec!["M106 P2 S178", "M106 P2 S0"]
    );
    assert_fan_after_z_before_segment_count(&output, 1, "M106 P2 S178");
}

#[tokio::test]
async fn invalid_auxiliary_fan_reaches_slice_error() {
    let err = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": "true",
        "additional_cooling_fan_speed": 70
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("auxiliary_fan"));
}

#[tokio::test]
async fn invalid_additional_cooling_fan_speed_reaches_slice_error() {
    let err = slice_auxiliary_fan_output(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 101
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("additional_cooling_fan_speed"));
}

async fn slice_auxiliary_fan_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn auxiliary_fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 P2 S"))
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
    let fan_index = lines[layer_index..]
        .iter()
        .position(|line| *line == fan_line)
        .map(|index| layer_index + index)
        .unwrap();
    let segment_count_index = lines[layer_index..]
        .iter()
        .position(|line| line.starts_with("; segment_count = "))
        .map(|index| layer_index + index)
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
