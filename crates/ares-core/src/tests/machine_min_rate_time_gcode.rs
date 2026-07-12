use super::*;
use serde_json::json;

#[tokio::test]
async fn machine_min_rates_reduce_print_time_sec_placeholder() {
    let baseline = output(json!({
        "machine_start_gcode": ";TIME [print_time_sec]"
    }))
    .await
    .unwrap();
    let clamped = output(json!({
        "machine_start_gcode": ";TIME [print_time_sec]",
        "machine_min_extruding_rate": [10_000.0, 0.0],
        "machine_min_travel_rate": [10_000.0, 0.0]
    }))
    .await
    .unwrap();

    assert!(time_value(&clamped) < time_value(&baseline));
}

#[tokio::test]
async fn invalid_machine_min_rate_reaches_slice_error_with_key() {
    let err = output(json!({
        "machine_start_gcode": ";TIME [print_time_sec]",
        "machine_min_travel_rate": ["NaN"]
    }))
    .await
    .unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("machine_min_travel_rate"))
    );
}

async fn output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn time_value(output: &str) -> f64 {
    output
        .lines()
        .find_map(|line| line.strip_prefix(";TIME "))
        .unwrap()
        .parse()
        .unwrap()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
