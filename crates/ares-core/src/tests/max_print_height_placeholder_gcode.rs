use super::*;

#[tokio::test]
async fn machine_start_max_print_height_renders_configured_value() {
    let output = slice_max_print_height_placeholder_output(json!({
        "machine_start_gcode": ";MAXH [max_print_height]",
        "printable_height": 256
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MAXH 256", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_max_print_height_defaults_to_orca_value() {
    let output = slice_max_print_height_placeholder_output(json!({
        "machine_start_gcode": ";MAXH [max_print_height]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MAXH 100", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_max_print_height_accepts_numeric_string_and_composes() {
    let output = slice_max_print_height_placeholder_output(json!({
        "machine_start_gcode": ";START [max_print_height] [first_layer_height] [z_offset] [num_extruders] [total_layer_count]",
        "printable_height": "260",
        "initial_layer_print_height": 0.24,
        "z_offset": 0.05,
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 260 0.24 0.05 2 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_max_print_height_rounds_like_orca_config_option_int() {
    let output = slice_max_print_height_placeholder_output(json!({
        "machine_start_gcode": ";MAXH [max_print_height]",
        "printable_height": 255.5
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MAXH 256", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_max_print_height_does_not_expand_in_layer_change_scope() {
    let output = slice_max_print_height_placeholder_output(json!({
        "layer_change_gcode": ";LC [max_print_height] [layer_num]",
        "printable_height": 256
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [max_print_height] 1", "; segment_count = 4");
}

#[tokio::test]
async fn machine_start_max_print_height_rejects_invalid_values() {
    for value in [json!(-0.1), json!("abc"), json!(["256"])] {
        let err = slice_max_print_height_placeholder_output(json!({
            "machine_start_gcode": ";MAXH [max_print_height]",
            "printable_height": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

async fn slice_max_print_height_placeholder_output(
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
