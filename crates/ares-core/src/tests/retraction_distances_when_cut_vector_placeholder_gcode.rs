use super::*;

#[tokio::test]
async fn machine_start_retraction_distances_when_cut_renders_configured_vector() {
    let output = slice_retraction_distances_when_cut_output(json!({
        "machine_start_gcode": ";CUTS [retraction_distances_when_cut]",
        "retraction_distances_when_cut": [10.0, 12.5, 18.0]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CUTS 10,12.5,18", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_cut_defaults_to_orca_vector() {
    let output = slice_retraction_distances_when_cut_output(json!({
        "machine_start_gcode": ";CUTS-DEFAULT [retraction_distances_when_cut]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CUTS-DEFAULT 18", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_cut_accepts_scalar_number() {
    let output = slice_retraction_distances_when_cut_output(json!({
        "machine_start_gcode": ";CUTS-SCALAR [retraction_distances_when_cut]",
        "retraction_distances_when_cut": 12.25
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";CUTS-SCALAR 12.25", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_retraction_distances_when_cut_accepts_string_and_composes() {
    let output = slice_retraction_distances_when_cut_output(json!({
        "machine_start_gcode": ";START [retraction_distances_when_cut] [retraction_distance_when_cut] [num_extruders]",
        "retraction_distances_when_cut": "10.75;18",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 10.75,18 10.75 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn retraction_distances_when_cut_stays_literal_in_layer_change_scope() {
    let output = slice_retraction_distances_when_cut_output(json!({
        "layer_change_gcode": ";LC [retraction_distances_when_cut] [layer_num]",
        "retraction_distances_when_cut": [10.0, 18.0]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [retraction_distances_when_cut] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_retraction_distances_when_cut_rejects_invalid_values() {
    for value in [
        json!(9.99),
        json!(18.01),
        json!([10.0, 18.01]),
        json!([]),
        json!(["bad"]),
        json!("NaN"),
        json!("inf"),
        json!("10,,18"),
        json!([12.0, "bad"]),
        json!(null),
        json!(true),
    ] {
        let err = slice_retraction_distances_when_cut_output(json!({
            "machine_start_gcode": ";CUTS [retraction_distances_when_cut]",
            "retraction_distances_when_cut": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("retraction_distances_when_cut"));
    }
}

async fn slice_retraction_distances_when_cut_output(
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
