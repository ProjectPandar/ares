use super::*;

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_renders_configured_vector() {
    let output = slice_bed_temperature_initial_layer_placeholder_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BED 35,65,45", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_defaults_to_selected_bed_type() {
    let output = slice_bed_temperature_initial_layer_placeholder_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
        "curr_bed_type": "Textured Cool Plate"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";BED 40", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_accepts_numeric_string_and_composes() {
    let output = slice_bed_temperature_initial_layer_placeholder_output(json!({
        "machine_start_gcode": ";START [bed_temperature_initial_layer] [first_layer_bed_temperature] [bed_temperature_initial_layer_single]",
        "cool_plate_temp_initial_layer": "41;55"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 41,55 41 55", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_suppresses_auto_bed_startup() {
    let output = slice_bed_temperature_initial_layer_placeholder_output(json!({
        "machine_start_gcode": "M140 S[bed_temperature_initial_layer]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await
    .unwrap();

    assert!(output.lines().any(|line| line == "M140 S35,65,45"));
    assert!(
        !output
            .lines()
            .any(|line| line == "M190 S65 ; set bed temperature and wait for it to be reached")
    );
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_stays_literal_in_layer_change() {
    let output = slice_bed_temperature_initial_layer_placeholder_output(json!({
        "layer_change_gcode": ";LC [bed_temperature_initial_layer] [layer_num]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [bed_temperature_initial_layer] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_rejects_invalid_values() {
    for extra in [
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
            "cool_plate_temp_initial_layer": -1
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
            "cool_plate_temp_initial_layer": []
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
            "cool_plate_temp_initial_layer": [35, "bad"]
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer]",
            "curr_bed_type": "Unknown Plate"
        }),
    ] {
        let err = slice_bed_temperature_initial_layer_placeholder_output(extra)
            .await
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

async fn slice_bed_temperature_initial_layer_placeholder_output(
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
