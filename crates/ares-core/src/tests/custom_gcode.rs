use super::*;

#[tokio::test]
async fn layer_change_gcode_emits_after_each_z_travel_before_layer_commands() {
    let output = slice_custom_gcode_output(json!({
        "layer_change_gcode": ";AFTER [layer_num] [layer_z]"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";AFTER 1 0.2", "G1 Z0.2 F7200");
    assert_line_before(&output, ";AFTER 1 0.2", "; segment_count = 4");
    assert_line_after(&output, ";AFTER 2 0.4", "G1 Z0.4 F7200");
    assert_line_before(&output, ";AFTER 2 0.4", "M106 S255");
}

#[tokio::test]
async fn layer_change_gcode_replaces_brace_layer_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "layer_change_gcode": ";LAYER-BRACE {layer_num} {layer_z} {max_layer_z}"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";LAYER-BRACE 1 0.2 0.2", "G1 Z0.2 F7200");
    assert_line_after(&output, ";LAYER-BRACE 2 0.4 0.4", "G1 Z0.4 F7200");
}

#[tokio::test]
async fn layer_change_gcode_replaces_bracket_layer_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "layer_change_gcode": ";LAYER-BRACKET [layer_num] [layer_z] [max_layer_z]"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";LAYER-BRACKET 1 0.2 0.2", "G1 Z0.2 F7200");
    assert_line_after(&output, ";LAYER-BRACKET 2 0.4 0.4", "G1 Z0.4 F7200");
}

#[tokio::test]
async fn layer_change_gcode_keeps_unknown_and_expression_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "layer_change_gcode": ";UNKNOWN {layer_num+1} [total_layer_count] {future_layer} [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_after(
        &output,
        ";UNKNOWN {layer_num+1} [total_layer_count] {future_layer} 1",
        "G1 Z0.2 F7200",
    );
}

#[tokio::test]
async fn layer_change_gcode_rejects_invalid_values() {
    let err = slice_custom_gcode_output(json!({
        "layer_change_gcode": ["; invalid"]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("layer_change_gcode"));
}

#[tokio::test]
async fn layer_change_gcode_absent_or_empty_is_noop() {
    let absent = slice_custom_gcode_output(json!({})).await.unwrap();
    let empty = slice_custom_gcode_output(json!({
        "layer_change_gcode": ""
    }))
    .await
    .unwrap();

    assert_eq!(without_option_count(&absent), without_option_count(&empty));
}

#[tokio::test]
async fn time_lapse_gcode_emits_after_each_z_travel_before_layer_change_gcode() {
    let output = slice_custom_gcode_output(json!({
        "time_lapse_gcode": ";TIMELAPSE [layer_num] [layer_z]",
        "layer_change_gcode": ";AFTER [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";TIMELAPSE 1 0.2", "G1 Z0.2 F7200");
    assert_line_before(&output, ";TIMELAPSE 1 0.2", ";AFTER 1");
    assert_line_after(&output, ";TIMELAPSE 2 0.4", "G1 Z0.4 F7200");
    assert_line_before(&output, ";TIMELAPSE 2 0.4", ";AFTER 2");
}

#[tokio::test]
async fn time_lapse_gcode_replaces_brace_layer_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "time_lapse_gcode": ";TL-BRACE {layer_num} {layer_z} {max_layer_z}"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";TL-BRACE 1 0.2 0.2", "G1 Z0.2 F7200");
    assert_line_after(&output, ";TL-BRACE 2 0.4 0.4", "G1 Z0.4 F7200");
}

#[tokio::test]
async fn time_lapse_gcode_replaces_bracket_layer_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "time_lapse_gcode": ";TL-BRACKET [layer_num] [layer_z] [max_layer_z]"
    }))
    .await
    .unwrap();

    assert_line_after(&output, ";TL-BRACKET 1 0.2 0.2", "G1 Z0.2 F7200");
    assert_line_after(&output, ";TL-BRACKET 2 0.4 0.4", "G1 Z0.4 F7200");
}

#[tokio::test]
async fn time_lapse_gcode_keeps_unknown_conditionals_and_expression_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "time_lapse_gcode": "{if timelapse_type == 0}\n;TL {layer_num+1} [future] [layer_num]\n{endif}"
    }))
    .await
    .unwrap();

    assert_line_after(&output, "{if timelapse_type == 0}", "G1 Z0.2 F7200");
    assert_line_after(
        &output,
        ";TL {layer_num+1} [future] 1",
        "{if timelapse_type == 0}",
    );
    assert_line_after(&output, "{endif}", ";TL {layer_num+1} [future] 1");
}

#[tokio::test]
async fn time_lapse_gcode_rejects_invalid_values() {
    let err = slice_custom_gcode_output(json!({
        "time_lapse_gcode": ["; invalid"]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("time_lapse_gcode"));
}

#[tokio::test]
async fn time_lapse_gcode_absent_or_empty_is_noop() {
    let absent = slice_custom_gcode_output(json!({})).await.unwrap();
    let empty = slice_custom_gcode_output(json!({
        "time_lapse_gcode": ""
    }))
    .await
    .unwrap();

    assert_eq!(without_option_count(&absent), without_option_count(&empty));
}

async fn slice_custom_gcode_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn without_option_count(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("; option_count = "))
        .collect::<Vec<_>>()
        .join("\n")
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

fn assert_line_after(output: &str, first: &str, second: &str) {
    assert_line_before(output, second, first);
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
