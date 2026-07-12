use super::*;

#[tokio::test]
async fn machine_start_gcode_renders_num_extruders_from_nozzle_diameters() {
    let output = slice_num_extruders_output(json!({
        "machine_start_gcode": ";EXTRUDERS [num_extruders]",
        "nozzle_diameter": ["0.4", "0.6", "0.8"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";EXTRUDERS 3", ";LAYER_CHANGE");
}

#[tokio::test]
async fn num_extruders_composes_with_total_layer_count_in_machine_start_gcode() {
    let output = slice_num_extruders_output(json!({
        "machine_start_gcode": ";START [num_extruders] [total_layer_count]",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";START 2 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn missing_nozzle_diameter_renders_default_num_extruders() {
    let output = slice_num_extruders_output(json!({
        "machine_start_gcode": ";DEFAULT-EXTRUDERS [num_extruders]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";DEFAULT-EXTRUDERS 1", ";LAYER_CHANGE");
}

#[tokio::test]
async fn num_extruders_does_not_expand_in_layer_change_scope() {
    let output = slice_num_extruders_output(json!({
        "layer_change_gcode": ";LC [num_extruders] [layer_num]",
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";LC [num_extruders] 1", "; segment_count = 4");
}

async fn slice_num_extruders_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
