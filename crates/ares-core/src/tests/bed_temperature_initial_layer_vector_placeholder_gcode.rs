use super::*;

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_vector_renders_empty_string() {
    let output = slice_bed_temperature_initial_layer_vector_output(json!({
        "machine_start_gcode": ";VECTOR <[bed_temperature_initial_layer_vector]>"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";VECTOR <>", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_vector_composes_with_bed_placeholders() {
    let output = slice_bed_temperature_initial_layer_vector_output(json!({
        "machine_start_gcode": ";START [bed_temperature_initial_layer] <[bed_temperature_initial_layer_vector]> [first_layer_bed_temperature] [bbl_bed_temperature_gcode]",
        "cool_plate_temp_initial_layer": [35, 65]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 35,65 <> 35 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn bed_temperature_initial_layer_vector_stays_literal_in_layer_change_scope() {
    let output = slice_bed_temperature_initial_layer_vector_output(json!({
        "layer_change_gcode": ";LC [bed_temperature_initial_layer_vector] [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(
        &output,
        ";LC [bed_temperature_initial_layer_vector] 1",
        "; segment_count = 4",
    );
}

async fn slice_bed_temperature_initial_layer_vector_output(
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
