use super::*;

#[tokio::test]
async fn filament_start_gcode_emits_after_machine_start_before_first_layer() {
    let output = slice_custom_gcode_output(json!({
        "machine_start_gcode": ";MACHINE-START",
        "filament_start_gcode": [";FILAMENT-START [filament_extruder_id]"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MACHINE-START", ";FILAMENT-START 0");
    assert_line_before(&output, ";FILAMENT-START 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn filament_start_gcode_emits_before_first_layer_without_machine_start_gcode() {
    let output = slice_custom_gcode_output(json!({
        "filament_start_gcode": ";FILAMENT-START"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FILAMENT-START", ";LAYER_CHANGE");
}

#[tokio::test]
async fn filament_start_gcode_replaces_brace_placeholder() {
    let output = slice_custom_gcode_output(json!({
        "filament_start_gcode": [";FS-BRACE {filament_extruder_id}"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FS-BRACE 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn filament_start_gcode_replaces_bracket_placeholder() {
    let output = slice_custom_gcode_output(json!({
        "filament_start_gcode": [";FS-BRACKET [filament_extruder_id]"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FS-BRACKET 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn filament_start_gcode_keeps_unknown_conditionals_and_expression_placeholders() {
    let output = slice_custom_gcode_output(json!({
        "filament_start_gcode": ["{if filament_extruder_id == 0}\n;FS {filament_extruder_id+1} [future] [filament_extruder_id]\n{endif}"]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        "{if filament_extruder_id == 0}",
        ";FS {filament_extruder_id+1} [future] 0",
    );
    assert_line_before(
        &output,
        ";FS {filament_extruder_id+1} [future] 0",
        "{endif}",
    );
    assert_line_before(&output, "{endif}", ";LAYER_CHANGE");
}

#[tokio::test]
async fn filament_start_gcode_rejects_invalid_values() {
    let err = slice_custom_gcode_output(json!({
        "filament_start_gcode": ["; ok", 7]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("filament_start_gcode"));
}

#[tokio::test]
async fn filament_start_gcode_absent_empty_and_empty_array_are_noop() {
    let absent = slice_custom_gcode_output(json!({})).await.unwrap();
    let empty = slice_custom_gcode_output(json!({
        "filament_start_gcode": ""
    }))
    .await
    .unwrap();
    let empty_array = slice_custom_gcode_output(json!({
        "filament_start_gcode": []
    }))
    .await
    .unwrap();

    assert_eq!(without_option_count(&absent), without_option_count(&empty));
    assert_eq!(
        without_option_count(&absent),
        without_option_count(&empty_array)
    );
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
