use super::*;

#[tokio::test]
async fn machine_start_gcode_min_vitrification_temperature_defaults_to_100() {
    let output = slice_temperature_vitrification_output(json!({
        "machine_start_gcode": ";SOFTEN [min_vitrification_temperature]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";SOFTEN 100", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_min_vitrification_temperature_uses_minimum_value() {
    let output = slice_temperature_vitrification_output(json!({
        "machine_start_gcode": ";SOFTEN [min_vitrification_temperature]",
        "temperature_vitrification": [105, 95, 110]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";SOFTEN 95", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_min_vitrification_temperature_accepts_separated_string() {
    let output = slice_temperature_vitrification_output(json!({
        "machine_start_gcode": ";SOFTEN [min_vitrification_temperature]",
        "temperature_vitrification": "102;98"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";SOFTEN 98", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_min_vitrification_temperature_rejects_invalid_values() {
    let err = slice_temperature_vitrification_output(json!({
        "machine_start_gcode": ";SOFTEN [min_vitrification_temperature]",
        "temperature_vitrification": -1
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("temperature_vitrification"));
}

#[tokio::test]
async fn rendered_min_vitrification_temperature_bed_command_suppresses_automatic_bed_startup() {
    let output = slice_temperature_vitrification_output(json!({
        "machine_start_gcode": "M140 S[min_vitrification_temperature]",
        "temperature_vitrification": [88, 92],
        "cool_plate_temp_initial_layer": 35
    }))
    .await
    .unwrap();

    assert!(
        !output
            .lines()
            .any(|line| line == "M190 S35 ; set bed temperature and wait for it to be reached")
    );
    assert_line_before(&output, "M140 S88", ";LAYER_CHANGE");
}

async fn slice_temperature_vitrification_output(
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
