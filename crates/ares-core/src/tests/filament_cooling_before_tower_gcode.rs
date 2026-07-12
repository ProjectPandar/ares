use super::*;

#[tokio::test]
async fn machine_start_gcode_filament_cooling_before_tower_defaults_to_10() {
    let output = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": ";COOL [filament_cooling_before_tower]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";COOL 10", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_filament_cooling_before_tower_renders_vector_values() {
    let output = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": ";COOL [filament_cooling_before_tower]",
        "filament_cooling_before_tower": [12, 7.5]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";COOL 12,7.5", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_filament_cooling_before_tower_accepts_separated_string() {
    let output = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": ";COOL [filament_cooling_before_tower]",
        "filament_cooling_before_tower": "13;8.25"
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";COOL 13,8.25", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_filament_cooling_before_tower_renders_nil_entries() {
    let output = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": ";COOL [filament_cooling_before_tower]",
        "filament_cooling_before_tower": [12, null, "nil", "7.5"]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";COOL 12,nil,nil,7.5", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_gcode_filament_cooling_before_tower_rejects_invalid_values() {
    let err = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": ";COOL [filament_cooling_before_tower]",
        "filament_cooling_before_tower": -1
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("filament_cooling_before_tower"));
}

#[tokio::test]
async fn rendered_filament_cooling_before_tower_nozzle_command_suppresses_automatic_startup() {
    let output = slice_filament_cooling_before_tower_output(json!({
        "machine_start_gcode": "M104 S[filament_cooling_before_tower]",
        "filament_cooling_before_tower": [215],
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

#[tokio::test]
async fn single_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": [10.0]
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 10")
    );
}

#[tokio::test]
async fn multiple_filament_cooling_before_tower_values_reach_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": [0.0, 10.0, 12.5]
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 0,10,12.5")
    );
}

#[tokio::test]
async fn scalar_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": 10.0
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 10")
    );
}

#[tokio::test]
async fn separated_string_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": "10;12.5"
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 10,12.5")
    );
}

#[tokio::test]
async fn mixed_nil_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": [null, 10.0, "nil", 12.5]
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = nil,10,nil,12.5")
    );
}

#[tokio::test]
async fn string_array_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": ["10", "12.5"]
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 10,12.5")
    );
}

#[tokio::test]
async fn zero_filament_cooling_before_tower_reaches_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": [0.0]
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = 0")
    );
}

#[tokio::test]
async fn empty_filament_cooling_before_tower_vector_reaches_empty_header_value() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": []
    }))
    .await
    .unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; filament_cooling_before_tower = ")
    );
}

#[tokio::test]
async fn all_nil_filament_cooling_before_tower_is_omitted_from_gcode_config_header() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_before_tower": [null, "nil"]
    }))
    .await
    .unwrap();

    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("; filament_cooling_before_tower = "))
    );
    assert!(output.lines().any(|line| line == "; generated by Ares"));
}

#[tokio::test]
async fn filament_cooling_before_tower_sits_between_minimal_purge_and_final_speed() {
    let output = filament_cooling_before_tower_header_output(json!({
        "filament_cooling_initial_speed": [2.2],
        "filament_minimal_purge_on_wipe_tower": [15.0],
        "filament_cooling_before_tower": [10.0],
        "filament_cooling_final_speed": [3.4]
    }))
    .await
    .unwrap();
    let lines = output.lines().collect::<Vec<_>>();
    let initial_index = lines
        .iter()
        .position(|line| *line == "; filament_cooling_initial_speed = 2.2")
        .unwrap();

    assert_eq!(
        lines.get(initial_index + 1),
        Some(&"; filament_minimal_purge_on_wipe_tower = 15")
    );
    assert_eq!(
        lines.get(initial_index + 2),
        Some(&"; filament_cooling_before_tower = 10")
    );
    assert_eq!(
        lines.get(initial_index + 3),
        Some(&"; filament_cooling_final_speed = 3.4")
    );
}

#[tokio::test]
async fn absent_filament_cooling_before_tower_preserves_header() {
    let output = filament_cooling_before_tower_header_output(json!({}))
        .await
        .unwrap();

    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("; filament_cooling_before_tower = "))
    );
    assert!(output.lines().any(|line| line == "; generated by Ares"));
}

#[tokio::test]
async fn invalid_filament_cooling_before_tower_values_reach_slice_error() {
    for value in [
        json!(-0.001),
        json!("bad"),
        json!("1;"),
        json!(true),
        json!({"value": 10.0}),
        json!([true]),
        json!(["bad"]),
        json!([-0.001]),
        json!([{"value": 10.0}]),
    ] {
        let err = filament_cooling_before_tower_header_output(json!({
            "filament_cooling_before_tower": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_cooling_before_tower"));
    }
}

#[tokio::test]
async fn invalid_filament_cooling_before_tower_is_rejected_when_header_is_skipped() {
    let err = filament_cooling_before_tower_header_output(json!({
        "thumbnails": "7x8/BTT_TFT",
        "filament_cooling_before_tower": [-0.001]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("filament_cooling_before_tower"));
}

async fn filament_cooling_before_tower_header_output(
    extra: serde_json::Value,
) -> Result<String, SliceError> {
    let mut options = serde_json::Map::new();
    options.insert("sparse_infill_density".to_owned(), json!(0));
    options.insert("filament_max_volumetric_speed".to_owned(), json!(0.0));
    for (key, value) in extra.as_object().unwrap() {
        options.insert(key.clone(), value.clone());
    }

    slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(serde_json::Value::Object(options)).unwrap(),
    )
    .await
    .map(|bytes| String::from_utf8(bytes).unwrap())
}

async fn slice_filament_cooling_before_tower_output(
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
