use super::*;

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_renders_nullable_vector() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "machine_start_gcode": ";EC-DISTS [retraction_distances_when_ec]",
        "retraction_distances_when_ec": [0.0, null, 10.0, 2.5]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-DISTS 0,nil,10,2.5", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_defaults_to_orca_vector() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "machine_start_gcode": ";EC-DISTS-DEFAULT [retraction_distances_when_ec]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-DISTS-DEFAULT 10", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_accepts_scalar_number() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "machine_start_gcode": ";EC-DISTS-SCALAR [retraction_distances_when_ec]",
        "retraction_distances_when_ec": 2.25
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-DISTS-SCALAR 2.25", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_accepts_scalar_null() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "machine_start_gcode": ";EC-DISTS-NULL [retraction_distances_when_ec]",
        "retraction_distances_when_ec": null
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-DISTS-NULL nil", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_accepts_serialized_string_and_composes() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "machine_start_gcode": ";START [retraction_distances_when_ec] [long_retractions_when_ec] [num_extruders]",
        "retraction_distances_when_ec": "nil, 2.5, 10",
        "long_retractions_when_ec": "nil, 1, 0",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START nil,2.5,10 nil,1,0 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn retraction_distances_when_ec_stays_literal_in_layer_change_scope() {
    let output = slice_retraction_distances_when_ec_output(json!({
        "layer_change_gcode": ";LC [retraction_distances_when_ec] [layer_num]",
        "retraction_distances_when_ec": [0.0, null, 10.0]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [retraction_distances_when_ec] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_retraction_distances_when_ec_rejects_invalid_values() {
    for value in [
        json!(-0.01),
        json!(10.01),
        json!([0.0, 10.01]),
        json!([]),
        json!(["bad"]),
        json!("NaN"),
        json!("inf"),
        json!("nil,,1"),
        json!("11"),
        json!("1;2"),
        json!([-1.0]),
        json!([null, "bad"]),
        json!(true),
    ] {
        let err = slice_retraction_distances_when_ec_output(json!({
            "machine_start_gcode": ";EC-DISTS [retraction_distances_when_ec]",
            "retraction_distances_when_ec": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("retraction_distances_when_ec"));
    }
}

async fn slice_retraction_distances_when_ec_output(
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
