use super::*;

#[tokio::test]
async fn default_head_wrap_detect_zone_renders_false_in_machine_start() {
    let output = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";HEAD 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn empty_head_wrap_detect_zone_renders_false_in_machine_start() {
    for value in [json!(""), json!("0x0")] {
        let output = head_wrap_output(json!({
            "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
            "head_wrap_detect_zone": value
        }))
        .await
        .unwrap();

        assert_line_before(&output, ";HEAD 0", ";LAYER_CHANGE");
    }
}

#[tokio::test]
async fn overlapping_head_wrap_detect_zone_renders_true_in_machine_start() {
    let output = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
        "head_wrap_detect_zone": "-3x-3,3x-3,3x3,-3x3"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";HEAD 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn non_overlapping_head_wrap_detect_zone_renders_false_in_machine_start() {
    let output = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
        "head_wrap_detect_zone": [[20, 20], [30, 20], [30, 30], [20, 30]]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";HEAD 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn invalid_head_wrap_detect_zone_errors_when_placeholder_is_used() {
    let err = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
        "head_wrap_detect_zone": "bad"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("head_wrap_detect_zone"));
}

#[tokio::test]
async fn unsupported_head_wrap_detect_zone_shape_errors_when_placeholder_is_used() {
    for value in [json!(true), json!([1, 2, 3])] {
        let err = head_wrap_output(json!({
            "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
            "head_wrap_detect_zone": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("head_wrap_detect_zone"));
    }
}

#[tokio::test]
async fn non_numeric_head_wrap_detect_zone_coordinate_errors_when_placeholder_is_used() {
    let err = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
        "head_wrap_detect_zone": [[0, 0], ["bad", 0], [1, 1]]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("head_wrap_detect_zone"));
}

#[tokio::test]
async fn non_finite_head_wrap_detect_zone_coordinate_errors_when_placeholder_is_used() {
    let err = head_wrap_output(json!({
        "machine_start_gcode": ";HEAD [in_head_wrap_detect_zone]",
        "head_wrap_detect_zone": "0x0,1x1,2xinf"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("head_wrap_detect_zone"));
}

#[tokio::test]
async fn head_wrap_detect_zone_placeholder_stays_literal_in_layer_change_scope() {
    let output = head_wrap_output(json!({
        "layer_change_gcode": ";LC [in_head_wrap_detect_zone] [layer_num]",
        "head_wrap_detect_zone": "-3x-3,3x-3,3x3,-3x3"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [in_head_wrap_detect_zone] 1",
        "; segment_count = 4",
    );
}

async fn head_wrap_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
