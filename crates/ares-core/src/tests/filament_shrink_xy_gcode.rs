use super::*;

#[tokio::test]
async fn filament_shrink_xy_reaches_named_gcode_move() {
    let output = filament_shrink_output(json!({
        "filament_shrink": "80%",
        "gcode_comments": true
    }))
    .await
    .unwrap();

    assert!(output.contains(";MOVE:travel:external_perimeter:-0.625,0"));
    assert!(output.contains("G1 X-0.625 Y0 F7200"));
}

#[tokio::test]
async fn invalid_filament_shrink_reaches_slice_error_with_key() {
    let err = filament_shrink_output(json!({
        "filament_shrink": "151%"
    }))
    .await
    .unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("filament_shrink"))
    );
}

async fn filament_shrink_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "skirt_loops": 0,
            "brim_width": 0.0,
            "seam_gap": 0,
            "slow_down_for_layer_cooling": false
        }),
        extra,
    );
    slice(super::square_pyramid_ascii_stl(), options)
        .await
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
