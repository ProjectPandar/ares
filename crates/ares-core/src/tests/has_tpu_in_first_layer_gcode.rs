use super::*;

#[tokio::test]
async fn machine_start_has_tpu_in_first_layer_renders_true_for_tpu() {
    let output = slice_has_tpu_in_first_layer_output(json!({
        "machine_start_gcode": ";TPU [has_tpu_in_first_layer]",
        "filament_type": ["TPU"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";TPU 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_has_tpu_in_first_layer_detects_tpu_in_any_configured_filament() {
    let output = slice_has_tpu_in_first_layer_output(json!({
        "machine_start_gcode": ";TPU [has_tpu_in_first_layer]",
        "filament_type": ["PLA", "TPU"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";TPU 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_has_tpu_in_first_layer_renders_false_without_tpu() {
    let output = slice_has_tpu_in_first_layer_output(json!({
        "machine_start_gcode": ";TPU [has_tpu_in_first_layer]",
        "filament_type": ["PLA", "PETG"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";TPU 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_has_tpu_in_first_layer_defaults_to_false() {
    let output = slice_has_tpu_in_first_layer_output(json!({
        "machine_start_gcode": ";TPU [has_tpu_in_first_layer]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";TPU 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn has_tpu_in_first_layer_stays_literal_in_layer_change_scope() {
    let output = slice_has_tpu_in_first_layer_output(json!({
        "layer_change_gcode": ";LC [has_tpu_in_first_layer] [layer_num]",
        "filament_type": ["TPU"]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [has_tpu_in_first_layer] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_has_tpu_in_first_layer_rejects_invalid_filament_type_vector() {
    for value in [json!("TPU"), json!([]), json!(["PLA", 7])] {
        let err = slice_has_tpu_in_first_layer_output(json!({
            "machine_start_gcode": ";TPU [has_tpu_in_first_layer]",
            "filament_type": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_type"));
    }
}

async fn slice_has_tpu_in_first_layer_output(
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
