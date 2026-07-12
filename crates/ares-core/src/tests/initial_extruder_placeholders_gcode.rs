use super::*;

#[tokio::test]
async fn machine_start_initial_extruder_placeholders_render_zero_ids() {
    let output = slice_initial_extruder_placeholders_output(json!({
        "machine_start_gcode": ";TOOLS [initial_tool] [initial_extruder] [current_extruder] [current_object_idx]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";TOOLS 0 0 0 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn initial_extruder_placeholders_compose_with_existing_start_placeholders() {
    let output = slice_initial_extruder_placeholders_output(json!({
        "machine_start_gcode": ";START [initial_tool] [current_extruder] [num_extruders] [total_layer_count]",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 0 0 2 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn initial_extruder_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_initial_extruder_placeholders_output(json!({
        "layer_change_gcode": ";LC [initial_tool] [initial_extruder] [current_extruder] [current_object_idx] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [initial_tool] [initial_extruder] [current_extruder] [current_object_idx] 1",
        "; segment_count = 4",
    );
}

async fn slice_initial_extruder_placeholders_output(
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
