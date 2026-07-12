use super::*;

#[tokio::test]
async fn machine_start_retract_length_renders_first_configured_value() {
    let output = slice_retract_length_output(json!({
        "machine_start_gcode": ";RETRACT [retract_length]",
        "retraction_length": [1.25, 9.0]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";RETRACT 1.25", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retract_length_defaults_to_orca_value() {
    let output = slice_retract_length_output(json!({
        "machine_start_gcode": ";DEFAULT-RETRACT [retract_length]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";DEFAULT-RETRACT 0.8", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retract_length_renders_zero_value() {
    let output = slice_retract_length_output(json!({
        "machine_start_gcode": ";ZERO-RETRACT [retract_length]",
        "retraction_length": 0
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";ZERO-RETRACT 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retract_length_composes_with_existing_placeholders() {
    let output = slice_retract_length_output(json!({
        "machine_start_gcode": ";START [retract_length] [num_extruders] [total_layer_count]",
        "retraction_length": "1.5,9",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 1.5 2 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retract_length_does_not_expand_in_layer_change_scope() {
    let output = slice_retract_length_output(json!({
        "layer_change_gcode": ";LC [retract_length] [layer_num]",
        "retraction_length": [1.25]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [retract_length] 1", "; segment_count = 4");
}

async fn slice_retract_length_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
