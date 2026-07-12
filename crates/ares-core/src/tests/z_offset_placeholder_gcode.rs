use super::*;

#[tokio::test]
async fn machine_start_z_offset_renders_configured_value() {
    let output = slice_z_offset_placeholder_output(json!({
        "machine_start_gcode": ";ZOFF [z_offset]",
        "z_offset": 0.15
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";ZOFF 0.15", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_z_offset_defaults_to_zero() {
    let output = slice_z_offset_placeholder_output(json!({
        "machine_start_gcode": ";ZOFF [z_offset]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";ZOFF 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_z_offset_renders_negative_value() {
    let output = slice_z_offset_placeholder_output(json!({
        "machine_start_gcode": ";ZOFF [z_offset]",
        "z_offset": -0.05
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";ZOFF -0.05", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_z_offset_accepts_numeric_string_and_composes() {
    let output = slice_z_offset_placeholder_output(json!({
        "machine_start_gcode": ";START [z_offset] [retract_length] [num_extruders] [total_layer_count]",
        "z_offset": "0.2",
        "retraction_length": 1.1,
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 0.2 1.1 2 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_z_offset_does_not_expand_in_layer_change_scope() {
    let output = slice_z_offset_placeholder_output(json!({
        "layer_change_gcode": ";LC [z_offset] [layer_num]",
        "z_offset": 0.15
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [z_offset] 1", "; segment_count = 4");
}

async fn slice_z_offset_placeholder_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
