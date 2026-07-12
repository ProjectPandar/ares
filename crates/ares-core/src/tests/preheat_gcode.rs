use super::*;

#[tokio::test]
async fn invalid_preheat_time_is_rejected_before_gcode_output() {
    let err = slice_preheat_result(json!({ "preheat_time": 120.001 }))
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("preheat_time"));
}

#[tokio::test]
async fn invalid_preheat_steps_is_rejected_before_gcode_output() {
    let err = slice_preheat_result(json!({ "preheat_steps": 0 }))
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("preheat_steps"));
}

#[tokio::test]
async fn valid_preheat_values_preserve_command_output() {
    let baseline = slice_preheat_output(json!({})).await;
    let configured = slice_preheat_output(json!({
        "preheat_time": 45.5,
        "preheat_steps": 3
    }))
    .await;

    assert_eq!(command_lines(&baseline), command_lines(&configured));
    assert_eq!(m104_lines(&baseline), m104_lines(&configured));
    assert!(!configured.lines().any(|line| line.starts_with("M104.1")));
}

async fn slice_preheat_result(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_preheat_output(extra: serde_json::Value) -> String {
    slice_preheat_result(extra).await.unwrap()
}

fn command_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect()
}

fn m104_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M104"))
        .collect()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
