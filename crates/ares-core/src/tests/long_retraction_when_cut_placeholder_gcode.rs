use super::*;

#[tokio::test]
async fn machine_start_long_retraction_when_cut_renders_first_configured_value() {
    let output = slice_long_retraction_when_cut_output(json!({
        "machine_start_gcode": ";LONG [long_retraction_when_cut]",
        "long_retractions_when_cut": [true, false]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LONG 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_cut_defaults_to_orca_false() {
    let output = slice_long_retraction_when_cut_output(json!({
        "machine_start_gcode": ";LONG-DEFAULT [long_retraction_when_cut]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LONG-DEFAULT 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_cut_accepts_comma_string_and_composes() {
    let output = slice_long_retraction_when_cut_output(json!({
        "machine_start_gcode": ";START [long_retraction_when_cut] [retraction_distance_when_cut] [num_extruders]",
        "long_retractions_when_cut": "1,0",
        "retraction_distances_when_cut": [12.5],
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 1 12.5 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_long_retraction_when_cut_renders_scalar_false() {
    let output = slice_long_retraction_when_cut_output(json!({
        "machine_start_gcode": ";LONG-FALSE [long_retraction_when_cut]",
        "long_retractions_when_cut": false
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LONG-FALSE 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn long_retraction_when_cut_stays_literal_in_layer_change_scope() {
    let output = slice_long_retraction_when_cut_output(json!({
        "layer_change_gcode": ";LC [long_retraction_when_cut] [layer_num]",
        "long_retractions_when_cut": true
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [long_retraction_when_cut] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_long_retraction_when_cut_rejects_invalid_values() {
    for value in [
        json!([]),
        json!([true, "bad"]),
        json!(null),
        json!(1),
        json!("true"),
        json!("false"),
        json!("nil"),
        json!(""),
        json!("1,,0"),
        json!("1,bad"),
        json!("1,2"),
        json!("1;0"),
    ] {
        let err = slice_long_retraction_when_cut_output(json!({
            "machine_start_gcode": ";LONG [long_retraction_when_cut]",
            "long_retractions_when_cut": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("long_retractions_when_cut"));
    }
}

async fn slice_long_retraction_when_cut_output(
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
