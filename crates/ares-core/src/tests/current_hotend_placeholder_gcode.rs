use super::*;

#[tokio::test]
async fn machine_start_current_hotend_defaults_to_initial_hotend_zero() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";HOTEND [current_hotend]"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";HOTEND 0",
        ";HOTEND [current_hotend]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn current_hotend_uses_zero_for_empty_printer_model() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";HOTEND [current_hotend]",
        "printer_model": ""
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";HOTEND 0",
        ";HOTEND [current_hotend]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn current_hotend_uses_zero_for_normal_printer_model() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";HOTEND [current_hotend]",
        "printer_model": "Bambu Lab X1 Carbon"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";HOTEND 0",
        ";HOTEND [current_hotend]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn current_hotend_uses_negative_one_for_bambu_x2d() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";HOTEND [current_hotend]",
        "printer_model": "Bambu Lab X2D"
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";HOTEND -1",
        ";HOTEND [current_hotend]",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn current_hotend_composes_with_current_extruder_and_nozzle_count() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";START [current_extruder] [current_hotend] [initial_tool] [num_extruders]",
        "nozzle_diameter": ["0.4", "0.6", "0.8"]
    }))
    .await
    .unwrap();

    assert_rendered_line_before(
        &output,
        ";START 0 0 0 3",
        ";START 0 [current_hotend] 0 3",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn current_hotend_stays_literal_in_layer_change_scope() {
    let output = slice_current_hotend_output(json!({
        "layer_change_gcode": ";LC [current_hotend] [current_extruder] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [current_hotend] [current_extruder] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn current_hotend_rejects_non_string_printer_model_when_used() {
    let err = slice_current_hotend_output(json!({
        "machine_start_gcode": ";HOTEND [current_hotend]",
        "printer_model": 7
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("printer_model"));
}

#[tokio::test]
async fn non_string_printer_model_is_ignored_when_current_hotend_is_unused() {
    let output = slice_current_hotend_output(json!({
        "machine_start_gcode": ";START [current_extruder]",
        "printer_model": 7
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 0", ";LAYER_CHANGE");
}

async fn slice_current_hotend_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
