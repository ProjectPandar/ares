use super::*;

#[tokio::test]
async fn default_slice_emits_first_layer_nozzle_temperature_before_first_layer() {
    let output = slice_temperature_output(json!({})).await;

    assert_line_before(
        &output,
        "M104 S200 ; set nozzle temperature",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn zero_first_layer_nozzle_temperature_suppresses_command() {
    let output = slice_temperature_output(json!({
        "nozzle_temperature_initial_layer": 0
    }))
    .await;

    assert_no_line_before(&output, "M104 S", ";LAYER_CHANGE");
    assert_no_line_before(&output, "G10 S", ";LAYER_CHANGE");
}

#[tokio::test]
async fn reprap_firmware_uses_g10_for_first_layer_nozzle_temperature() {
    let output = slice_temperature_output(json!({
        "gcode_flavor": "reprapfirmware",
        "nozzle_temperature_initial_layer": 215
    }))
    .await;

    assert_line_before(
        &output,
        "G10 S215 ; set nozzle temperature",
        ";LAYER_CHANGE",
    );
    assert!(!output.lines().any(|line| line.starts_with("M104 S215")));
}

#[tokio::test]
async fn klipper_skips_first_layer_nozzle_temperature_startup_command() {
    let output = slice_temperature_output(json!({
        "gcode_flavor": "klipper",
        "nozzle_temperature_initial_layer": 215
    }))
    .await;

    assert!(!output.lines().any(|line| line.starts_with("M104 S215")));
    assert!(!output.lines().any(|line| line.starts_with("G10 S215")));
}

#[tokio::test]
async fn machine_start_gcode_nozzle_command_suppresses_automatic_nozzle_startup_command() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M104 S215",
        "nozzle_temperature_initial_layer": 200
    }))
    .await;

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("M104 S"))
            .collect::<Vec<_>>(),
        vec!["M104 S215"]
    );
}

#[tokio::test]
async fn machine_start_gcode_nozzle_wait_command_suppresses_automatic_nozzle_startup_command() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M109 S215",
        "nozzle_temperature_initial_layer": 200
    }))
    .await;

    assert!(
        !output
            .lines()
            .any(|line| line == "M104 S200 ; set nozzle temperature")
    );
    assert!(output.lines().any(|line| line == "M109 S215"));
}

#[tokio::test]
async fn reprap_machine_start_gcode_g10_suppresses_automatic_nozzle_startup_command() {
    let output = slice_temperature_output(json!({
        "gcode_flavor": "reprapfirmware",
        "machine_start_gcode": "G10 S215",
        "nozzle_temperature_initial_layer": 200
    }))
    .await;

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("G10 S"))
            .collect::<Vec<_>>(),
        vec!["G10 S215"]
    );
}

#[tokio::test]
async fn machine_start_first_layer_temperature_renders_configured_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";NOZZLE [first_layer_temperature]",
        "nozzle_temperature_initial_layer": 215
    }))
    .await;

    assert_line_before(&output, ";NOZZLE 215", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_temperature_defaults_to_orca_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";NOZZLE [first_layer_temperature]"
    }))
    .await;

    assert_line_before(&output, ";NOZZLE 200", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_temperature_uses_initial_extruder_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";NOZZLE [first_layer_temperature]",
        "nozzle_temperature_initial_layer": [210, 230]
    }))
    .await;

    assert_line_before(&output, ";NOZZLE 210", ";LAYER_CHANGE");
}

#[tokio::test]
async fn machine_start_first_layer_temperature_accepts_numeric_string_and_composes() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";START [first_layer_temperature] [max_print_height] [first_layer_height] [z_offset]",
        "nozzle_temperature_initial_layer": "225;235",
        "printable_height": 256,
        "initial_layer_print_height": 0.24,
        "z_offset": 0.05
    }))
    .await;

    assert_line_before(&output, ";START 225 256 0.24 0.05", ";LAYER_CHANGE");
}

#[tokio::test]
async fn first_layer_temperature_does_not_expand_in_layer_change_scope() {
    let output = slice_temperature_output(json!({
        "layer_change_gcode": ";LC [first_layer_temperature] [layer_num]",
        "nozzle_temperature_initial_layer": 215
    }))
    .await;

    assert_line_before(
        &output,
        ";LC [first_layer_temperature] 1",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn machine_start_first_layer_temperature_rejects_invalid_values() {
    for value in [json!(-1), json!("abc"), json!([]), json!([215, "bad"])] {
        let err = slice_temperature_result(json!({
            "machine_start_gcode": ";NOZZLE [first_layer_temperature]",
            "nozzle_temperature_initial_layer": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[tokio::test]
async fn temperature_placeholder_machine_start_renders_configured_scalar() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";TEMP [temperature]",
        "nozzle_temperature": 215
    }))
    .await;

    assert_line_before(&output, ";TEMP 215", ";LAYER_CHANGE");
}

#[tokio::test]
async fn temperature_placeholder_machine_start_defaults_to_orca_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";TEMP-DEFAULT [temperature]"
    }))
    .await;

    assert_line_before(&output, ";TEMP-DEFAULT 200", ";LAYER_CHANGE");
}

#[tokio::test]
async fn temperature_placeholder_machine_start_uses_initial_extruder_value() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";TEMP-FIRST [temperature]",
        "nozzle_temperature": [215, 235]
    }))
    .await;

    assert_line_before(&output, ";TEMP-FIRST 215", ";LAYER_CHANGE");
}

#[tokio::test]
async fn temperature_placeholder_machine_start_accepts_serialized_string_and_composes() {
    let output = slice_temperature_output(json!({
        "machine_start_gcode": ";START [temperature] [first_layer_temperature] [num_extruders]",
        "nozzle_temperature": "225;235",
        "nozzle_temperature_initial_layer": [205, 210],
        "nozzle_diameter": ["0.4", "0.6"]
    }))
    .await;

    assert_line_before(&output, ";START 225 205 2", ";LAYER_CHANGE");
}

#[tokio::test]
async fn temperature_placeholder_machine_start_suppresses_automatic_nozzle_startup_after_rendering()
{
    let output = slice_temperature_output(json!({
        "machine_start_gcode": "M104 S[temperature]",
        "nozzle_temperature": 215,
        "nozzle_temperature_initial_layer": 200
    }))
    .await;

    assert_eq!(
        output
            .lines()
            .take_while(|line| *line != ";LAYER_CHANGE")
            .filter(|line| line.starts_with("M104 S"))
            .collect::<Vec<_>>(),
        vec!["M104 S215"]
    );
}

#[tokio::test]
async fn temperature_placeholder_stays_literal_in_layer_change_scope() {
    let output = slice_temperature_output(json!({
        "layer_change_gcode": ";LC [temperature] [layer_num]",
        "nozzle_temperature": 215
    }))
    .await;

    assert_line_before(&output, ";LC [temperature] 1", "; segment_count = 4");
}

#[tokio::test]
async fn temperature_placeholder_machine_start_rejects_invalid_nozzle_temperature() {
    for value in [json!(-1), json!("abc"), json!([]), json!([215, "bad"])] {
        let err = slice_temperature_result(json!({
            "machine_start_gcode": ";TEMP [temperature]",
            "nozzle_temperature": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("nozzle_temperature"));
    }
}

#[tokio::test]
async fn nozzle_temperature_range_rejects_incompatible_multi_filament_slice() {
    let err = slice_temperature_result(json!({
        "nozzle_diameter": ["0.4", "0.4"],
        "filament_diameter": ["1.75", "1.75"],
        "filament_type": ["PLA", "ABS"],
        "nozzle_temperature": [200, 260],
        "nozzle_temperature_range_low": [190, 250],
        "nozzle_temperature_range_high": [230, 280]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_temperature"));
    assert!(err.to_string().contains("nozzle_temperature_range_low"));
    assert!(err.to_string().contains("nozzle_temperature_range_high"));
}

#[tokio::test]
async fn nozzle_temperature_range_validates_before_model_loading() {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0
        }),
        json!({
            "nozzle_diameter": ["0.4", "0.4"],
            "filament_diameter": ["1.75", "1.75"],
            "filament_type": ["PLA", "ABS"],
            "nozzle_temperature": [200, 260],
            "nozzle_temperature_range_low": [190, 250],
            "nozzle_temperature_range_high": [230, 280]
        }),
    );

    let err = slice(b"not a model", options).await.unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_temperature"));
    assert!(
        !err.to_string()
            .contains("unsupported or malformed model input")
    );
}

#[tokio::test]
async fn nozzle_temperature_range_allows_mutually_compatible_multi_filament_slice() {
    let output = slice_temperature_output(json!({
        "nozzle_diameter": ["0.4", "0.4"],
        "filament_diameter": ["1.75", "1.75"],
        "filament_type": ["PLA", "PETG"],
        "nozzle_temperature": [210, 220],
        "nozzle_temperature_range_low": [190, 205],
        "nozzle_temperature_range_high": [230, 240]
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "; nozzle_diameter = 0.4,0.4")
    );
    assert!(output.lines().any(|line| line == ";LAYER_CHANGE"));
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

fn assert_no_line_before(output: &str, forbidden_prefix: &str, marker: &str) {
    for line in output.lines().take_while(|line| *line != marker) {
        assert!(
            !line.starts_with(forbidden_prefix),
            "unexpected {forbidden_prefix} before {marker}: {line}"
        );
    }
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
