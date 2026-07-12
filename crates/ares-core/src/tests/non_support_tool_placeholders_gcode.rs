use super::*;

#[tokio::test]
async fn machine_start_non_support_tool_placeholders_render_current_initial_tool() {
    let output = slice_non_support_tool_placeholders_output(json!({
        "machine_start_gcode": ";NOSUPPORT [first_non_support_tools] [first_non_support_filaments] [initial_no_support_tool] [initial_no_support_extruder] [initial_no_support_hotend]"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";NOSUPPORT 0 0 0 0 0",
        ";NOSUPPORT [first_non_support_tools] [first_non_support_filaments] [initial_no_support_tool] [initial_no_support_extruder] [initial_no_support_hotend]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn non_support_tool_placeholders_keep_unused_nozzles_marked_negative_one() {
    let output = slice_non_support_tool_placeholders_output(json!({
        "machine_start_gcode": ";NOSUPPORT [num_extruders] [first_tools] [first_non_support_tools] [first_non_support_filaments] [initial_tool] [initial_no_support_tool] [total_layer_count]",
        "nozzle_diameter": ["0.4", "0.6", "0.8"]
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";NOSUPPORT 3 0,-1,-1 0,-1,-1 0,-1,-1 0 0 2",
        ";NOSUPPORT 3 0,-1,-1 [first_non_support_tools] [first_non_support_filaments] 0 [initial_no_support_tool] 2",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn non_support_tool_placeholders_stay_literal_in_layer_change_scope() {
    let output = slice_non_support_tool_placeholders_output(json!({
        "layer_change_gcode": ";LC [first_non_support_tools] [first_non_support_filaments] [initial_no_support_tool] [initial_no_support_extruder] [initial_no_support_hotend] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [first_non_support_tools] [first_non_support_filaments] [initial_no_support_tool] [initial_no_support_extruder] [initial_no_support_hotend] 1",
        "; segment_count = 4",
    );
}

async fn slice_non_support_tool_placeholders_output(
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

fn assert_rendered_line_before(output: &str, rendered: &str, literal: &str, next: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let rendered_index = lines.iter().position(|line| *line == rendered);
    let literal_present = lines.contains(&literal);

    assert!(
        rendered_index.is_some(),
        "missing rendered line {rendered:?}; literal present: {literal_present}"
    );

    let next_index = lines.iter().position(|line| *line == next).unwrap();
    assert!(
        rendered_index.unwrap() < next_index,
        "{} !< {next_index}",
        rendered_index.unwrap()
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
