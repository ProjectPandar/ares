use super::*;

#[tokio::test]
async fn machine_start_long_retraction_when_ec_uses_first_configured_value() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";EC-LONG [long_retraction_when_ec]",
        "long_retractions_when_ec": [false, true]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-LONG 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_defaults_to_orca_false() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";EC-LONG-DEFAULT [long_retraction_when_ec]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-LONG-DEFAULT 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_accepts_scalar_bool() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";EC-LONG-SCALAR [long_retraction_when_ec]",
        "long_retractions_when_ec": true
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-LONG-SCALAR 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_accepts_scalar_false() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";EC-LONG-FALSE [long_retraction_when_ec]",
        "long_retractions_when_ec": false
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-LONG-FALSE 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_renders_scalar_null_as_true() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";EC-LONG-NULL [long_retraction_when_ec]",
        "long_retractions_when_ec": null
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EC-LONG-NULL 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_accepts_serialized_string_and_composes() {
    let output = slice_long_retraction_when_ec_output(json!({
        "machine_start_gcode": ";START [long_retraction_when_ec] [long_retractions_when_ec] [num_extruders]",
        "long_retractions_when_ec": "nil, 1, 0",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 1 nil,1,0 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn long_retraction_when_ec_stays_literal_in_layer_change_scope() {
    let output = slice_long_retraction_when_ec_output(json!({
        "layer_change_gcode": ";LC [long_retraction_when_ec] [layer_num]",
        "long_retractions_when_ec": [true, false]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [long_retraction_when_ec] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_long_retraction_when_ec_rejects_invalid_values() {
    for value in [
        json!([]),
        json!([true, "bad"]),
        json!(1),
        json!("true"),
        json!("false"),
        json!(""),
        json!("1,,0"),
        json!("1,bad"),
        json!("1,2"),
        json!("1;0"),
    ] {
        let err = slice_long_retraction_when_ec_output(json!({
            "machine_start_gcode": ";EC-LONG [long_retraction_when_ec]",
            "long_retractions_when_ec": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("long_retractions_when_ec"));
    }
}

async fn slice_long_retraction_when_ec_output(
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
