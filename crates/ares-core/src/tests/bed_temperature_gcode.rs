use super::*;

#[tokio::test]
async fn default_slice_emits_first_layer_bed_temperature_before_nozzle_temperature() {
    let output = slice_temperature_output(json!({})).await;

    assert_line_before(
        &output,
        "M190 S35 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
    assert_line_before(
        &output,
        "M190 S35 ; set bed temperature and wait for it to be reached",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn selected_bed_type_changes_default_first_layer_bed_temperature() {
    let output = slice_temperature_output(json!({
        "curr_bed_type": "Textured Cool Plate"
    }))
    .await;

    assert_line_before(
        &output,
        "M190 S40 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn selected_bed_temperature_key_overrides_default() {
    let output = slice_temperature_output(json!({
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": 67
    }))
    .await;

    assert_line_before(
        &output,
        "M190 S67 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn default_bed_temperature_formula_uses_highest_first_layer_temperature() {
    let output = slice_temperature_output(json!({
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(
        &output,
        "M190 S65 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn first_filament_bed_temperature_formula_uses_first_layer_vector_entry() {
    let output = slice_temperature_output(json!({
        "bed_temperature_formula": "by_first_filament",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(
        &output,
        "M190 S35 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn zero_first_layer_bed_temperature_emits_zero_wait_command() {
    let output = slice_temperature_output(json!({
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": 0
    }))
    .await;

    assert_line_before(
        &output,
        "M190 S0 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn klipper_skips_first_layer_bed_temperature_startup_command() {
    let output = slice_temperature_output(json!({
        "gcode_flavor": "klipper",
        "cool_plate_temp_initial_layer": 35
    }))
    .await;

    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("M190 S") || line.starts_with("M140 S"))
    );
}

#[tokio::test]
async fn machine_start_gcode_bed_wait_command_suppresses_automatic_bed_startup_command() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M190 S70",
        "cool_plate_temp_initial_layer": 35
    }))
    .await;

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("M190 S"))
            .collect::<Vec<_>>(),
        vec!["M190 S70"]
    );
}

#[tokio::test]
async fn machine_start_gcode_bed_non_wait_command_suppresses_automatic_bed_startup_command() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M140 S70",
        "cool_plate_temp_initial_layer": 35
    }))
    .await;

    assert!(
        !output
            .lines()
            .any(|line| line == "M190 S35 ; set bed temperature and wait for it to be reached")
    );
    assert!(output.lines().any(|line| line == "M140 S70"));
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_renders_configured_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [first_layer_bed_temperature]",
        "cool_plate_temp_initial_layer": 47
    }))
    .await;

    assert_line_before(&output, ";BED 47", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_defaults_to_selected_bed_type() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [first_layer_bed_temperature]",
        "curr_bed_type": "Textured Cool Plate"
    }))
    .await;

    assert_line_before(&output, ";BED 40", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_uses_initial_extruder_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [first_layer_bed_temperature]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(&output, ";BED 35", ";LAYER_CHANGE");
    assert_line_before(
        &output,
        "M190 S65 ; set bed temperature and wait for it to be reached",
        "M104 S200 ; set nozzle temperature",
    );
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_accepts_numeric_string_and_composes() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";START [first_layer_bed_temperature] [first_layer_temperature] [max_print_height] [first_layer_height]",
        "cool_plate_temp_initial_layer": "41;55",
        "nozzle_temperature_initial_layer": 215,
        "printable_height": 256,
        "initial_layer_print_height": 0.24
    }))
    .await;

    assert_line_before(&output, ";START 41 215 256 0.24", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_does_not_expand_in_layer_change_scope() {
    let output = slice_temperature_output(json!({
        "layer_change_gcode": ";LC [first_layer_bed_temperature] [layer_num]",
        "cool_plate_temp_initial_layer": 47
    }))
    .await;

    assert_line_before(
        &output,
        ";LC [first_layer_bed_temperature] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_first_layer_bed_temperature_rejects_invalid_values() {
    for extra in [
        json!({
            "machine_start_gcode": ";BED [first_layer_bed_temperature]",
            "cool_plate_temp_initial_layer": -1
        }),
        json!({
            "machine_start_gcode": ";BED [first_layer_bed_temperature]",
            "cool_plate_temp_initial_layer": "abc"
        }),
        json!({
            "machine_start_gcode": ";BED [first_layer_bed_temperature]",
            "cool_plate_temp_initial_layer": []
        }),
        json!({
            "machine_start_gcode": ";BED [first_layer_bed_temperature]",
            "cool_plate_temp_initial_layer": [35, "bad"]
        }),
        json!({
            "machine_start_gcode": ";BED [first_layer_bed_temperature]",
            "curr_bed_type": "Unknown Plate"
        }),
    ] {
        let err = slice_temperature_result(extra).await.unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_uses_highest_formula_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(&output, ";BED 65", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_uses_first_filament_formula_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
        "bed_temperature_formula": "by_first_filament",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(&output, ";BED 35", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_defaults_to_selected_bed_type() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
        "curr_bed_type": "Textured Cool Plate"
    }))
    .await;

    assert_line_before(&output, ";BED 40", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_renders_zero_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": 0
    }))
    .await;

    assert_line_before(&output, ";BED 0", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_suppresses_auto_bed_startup() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M140 S[bed_temperature_initial_layer_single]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert!(output.lines().any(|line| line == "M140 S65"));
    assert!(
        !output
            .lines()
            .any(|line| line == "M190 S65 ; set bed temperature and wait for it to be reached")
    );
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_stays_literal_in_layer_change() {
    let output = slice_temperature_output(json!({
        "layer_change_gcode": ";LC [bed_temperature_initial_layer_single] [layer_num]",
        "cool_plate_temp_initial_layer": [35, 65, 45]
    }))
    .await;

    assert_line_before(
        &output,
        ";LC [bed_temperature_initial_layer_single] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_bed_temperature_initial_layer_single_rejects_invalid_values() {
    for extra in [
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
            "cool_plate_temp_initial_layer": -1
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
            "cool_plate_temp_initial_layer": []
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
            "bed_temperature_formula": "unknown",
            "cool_plate_temp_initial_layer": 35
        }),
        json!({
            "machine_start_gcode": ";BED [bed_temperature_initial_layer_single]",
            "curr_bed_type": "Unknown Plate"
        }),
    ] {
        let err = slice_temperature_result(extra).await.unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

async fn slice_temperature_result(extra: serde_json::Value) -> Result<String, SliceError> {
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

async fn slice_temperature_output(extra: serde_json::Value) -> String {
    slice_temperature_result(extra).await.unwrap()
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
