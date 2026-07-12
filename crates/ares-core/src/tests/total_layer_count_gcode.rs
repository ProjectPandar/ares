use super::*;

#[tokio::test]
async fn machine_start_gcode_renders_total_layer_count_before_first_layer() {
    let output = slice_total_layer_count_output(json!({
        "machine_start_gcode": ";LAYERS [total_layer_count]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LAYERS 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn total_layer_count_composes_with_existing_machine_start_placeholders() {
    let output = slice_total_layer_count_output(json!({
        "machine_start_gcode": ";START [total_layer_count] [min_vitrification_temperature]",
        "temperature_vitrification": [105, 95]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 2 95", ";LAYER_CHANGE");
}

#[tokio::test]
async fn total_layer_count_does_not_expand_in_layer_change_scope() {
    let output = slice_total_layer_count_output(json!({
        "layer_change_gcode": ";LC [total_layer_count] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [total_layer_count] 1", "; segment_count = 4");
}

async fn slice_total_layer_count_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
