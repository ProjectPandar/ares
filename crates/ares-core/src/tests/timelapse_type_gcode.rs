use super::*;

#[tokio::test]
async fn invalid_timelapse_type_is_rejected_before_gcode_output() {
    let err = slice_timelapse_type_result(json!({ "timelapse_type": "smooth" }))
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("timelapse_type"));
}

#[tokio::test]
async fn valid_timelapse_types_preserve_command_output() {
    let baseline = slice_timelapse_type_output(json!({})).await;

    for extra in [
        json!({ "timelapse_type": "0" }),
        json!({ "timelapse_type": "1" }),
        json!({ "timelapse_type": "2" }),
        json!({ "timelapse_no_toolhead": "2" }),
    ] {
        let configured = slice_timelapse_type_output(extra).await;

        assert_eq!(command_lines(&baseline), command_lines(&configured));
    }
}

#[tokio::test]
async fn timelapse_type_does_not_change_custom_time_lapse_gcode_output() {
    let custom = json!({ "time_lapse_gcode": ";TL-CUSTOM [layer_num] [layer_z]" });
    let baseline = slice_timelapse_type_output(custom.clone()).await;
    let configured =
        slice_timelapse_type_output(merged_json(custom, json!({ "timelapse_type": "1" }))).await;

    assert!(configured.contains(";TL-CUSTOM 1 0.2"));
    assert_eq!(command_lines(&baseline), command_lines(&configured));
}

async fn slice_timelapse_type_result(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_timelapse_type_output(extra: serde_json::Value) -> String {
    slice_timelapse_type_result(extra).await.unwrap()
}

fn command_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    serde_json::from_value(merged_json(base, extra)).unwrap()
}

fn merged_json(base: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::Value::Object(base)
}
