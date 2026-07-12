use super::*;

#[tokio::test]
async fn flush_placeholders_render_orca_defaults_in_machine_start_gcode() {
    let output = slice_flush_placeholders_output(json!({
        "machine_start_gcode": ";FLUSHV [flush_volumetric_speeds]\n;FLUSHT [flush_temperatures]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FLUSHV 2", ";LAYER_CHANGE");
    assert_line_before(&output, ";FLUSHT 240", ";LAYER_CHANGE");
}

#[tokio::test]
async fn flush_placeholders_render_zero_fallbacks_and_preserve_non_zero_values() {
    let output = slice_flush_placeholders_output(json!({
        "machine_start_gcode": ";FLUSHV [flush_volumetric_speeds]\n;FLUSHT [flush_temperatures]",
        "filament_flush_volumetric_speed": [0, 4.5],
        "filament_max_volumetric_speed": [2, 8],
        "filament_flush_temp": [0, 245],
        "nozzle_temperature_range_high": [260, 270]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FLUSHV 2,4.5", ";LAYER_CHANGE");
    assert_line_before(&output, ";FLUSHT 260,245", ";LAYER_CHANGE");
}

#[tokio::test]
async fn flush_placeholders_reuse_first_fallback_value_in_gcode() {
    let output = slice_flush_placeholders_output(json!({
        "machine_start_gcode": ";FLUSHV [flush_volumetric_speeds]\n;FLUSHT [flush_temperatures]",
        "filament_flush_volumetric_speed": [0, 0],
        "filament_max_volumetric_speed": [3.5],
        "filament_flush_temp": [0, 0],
        "nozzle_temperature_range_high": [255]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";FLUSHV 3.5,3.5", ";LAYER_CHANGE");
    assert_line_before(&output, ";FLUSHT 255,255", ";LAYER_CHANGE");
}

#[tokio::test]
async fn invalid_flush_placeholder_values_reach_slice_error() {
    for (key, value) in [
        ("filament_flush_volumetric_speed", json!(200.1)),
        ("filament_flush_temp", json!(1501)),
        ("nozzle_temperature_range_high", json!("bad")),
    ] {
        let err = slice_flush_placeholders_output(json!({
            "machine_start_gcode": ";FLUSH [flush_volumetric_speeds] [flush_temperatures]",
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[tokio::test]
async fn rendered_flush_temperature_nozzle_command_suppresses_automatic_nozzle_startup() {
    let output = slice_flush_placeholders_output(json!({
        "machine_start_gcode": "M104 S[flush_temperatures]",
        "filament_flush_temp": [0],
        "nozzle_temperature_range_high": [215],
        "nozzle_temperature_initial_layer": 200
    }))
    .await
    .unwrap();

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("M104 S"))
            .collect::<Vec<_>>(),
        vec!["M104 S215"]
    );
    assert_line_before(&output, "M104 S215", ";LAYER_CHANGE");
}

async fn slice_flush_placeholders_output(extra: serde_json::Value) -> Result<String, SliceError> {
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
