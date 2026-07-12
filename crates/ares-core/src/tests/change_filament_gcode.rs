use super::*;

#[tokio::test]
async fn invalid_change_filament_gcode_is_rejected_before_gcode_output() {
    let err = slice_change_filament_result(json!({
        "change_filament_gcode": ["M600"]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("change_filament_gcode"));
}

#[tokio::test]
async fn valid_change_filament_gcode_preserves_command_output() {
    let baseline = slice_change_filament_output(json!({})).await;

    for extra in [
        json!({ "change_filament_gcode": "" }),
        json!({ "change_filament_gcode": "M600" }),
        json!({ "tool_change_gcode": "M600" }),
    ] {
        let configured = slice_change_filament_output(extra).await;
        assert_eq!(command_lines(&baseline), command_lines(&configured));
    }
}

#[tokio::test]
async fn change_filament_gcode_is_not_inserted_without_toolchange_support() {
    let output = slice_change_filament_output(json!({
        "change_filament_gcode": "M600"
    }))
    .await;

    assert!(!output.contains("M600"));
}

async fn slice_change_filament_result(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_change_filament_output(extra: serde_json::Value) -> String {
    slice_change_filament_result(extra).await.unwrap()
}

fn command_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
