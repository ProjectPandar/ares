use super::*;

#[tokio::test]
async fn invalid_single_extruder_multi_material_is_rejected_before_gcode_output() {
    let err = slice_filament_change_result(json!({
        "single_extruder_multi_material": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("single_extruder_multi_material"));
}

#[tokio::test]
async fn invalid_manual_filament_change_is_rejected_before_gcode_output() {
    let err = slice_filament_change_result(json!({
        "manual_filament_change": "false"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("manual_filament_change"));
}

#[tokio::test]
async fn valid_filament_change_flags_preserve_command_output() {
    let baseline = slice_filament_change_output(json!({})).await;

    for extra in [
        json!({ "single_extruder_multi_material": true, "manual_filament_change": false }),
        json!({ "single_extruder_multi_material": true, "manual_filament_change": true }),
        json!({ "single_extruder_multi_material": false, "manual_filament_change": false }),
        json!({ "single_extruder_multi_material": false, "manual_filament_change": true }),
    ] {
        let configured = slice_filament_change_output(extra).await;

        assert_eq!(command_lines(&baseline), command_lines(&configured));
    }
}

async fn slice_filament_change_result(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_filament_change_output(extra: serde_json::Value) -> String {
    slice_filament_change_result(extra).await.unwrap()
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
