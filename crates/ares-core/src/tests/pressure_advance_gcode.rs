use super::*;

#[tokio::test]
async fn default_slice_does_not_emit_pressure_advance_command() {
    let output = pressure_advance_output(json!({})).await.unwrap();

    assert_no_pressure_advance(&output);
}

#[tokio::test]
async fn disabled_pressure_advance_suppresses_configured_value() {
    let output = pressure_advance_output(json!({
        "enable_pressure_advance": [false],
        "pressure_advance": [0.045]
    }))
    .await
    .unwrap();

    assert_no_pressure_advance(&output);
}

#[tokio::test]
async fn marlin_pressure_advance_emits_startup_m900() {
    let output = pressure_advance_output(json!({
        "enable_pressure_advance": [true],
        "pressure_advance": [0.04567]
    }))
    .await
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec!["M900 K0.0457; Override pressure advance value"]
    );
}

#[tokio::test]
async fn enabled_pressure_advance_uses_default_value() {
    let output = pressure_advance_output(json!({
        "enable_pressure_advance": true
    }))
    .await
    .unwrap();

    assert_eq!(
        pressure_advance_lines(&output),
        vec!["M900 K0.02; Override pressure advance value"]
    );
}

#[tokio::test]
async fn flavor_specific_pressure_advance_commands_are_emitted() {
    let cases = [
        (
            "klipper",
            "SET_PRESSURE_ADVANCE ADVANCE=0.12; Override pressure advance value",
        ),
        (
            "reprapfirmware",
            "M572 D0 S0.12; Override pressure advance value",
        ),
        (
            "repetier",
            "M233 X0.12 Y0.12 ; Override pressure advance value",
        ),
        ("marlin2", "M900 K0.12; Override pressure advance value"),
    ];

    for (flavor, expected) in cases {
        let output = pressure_advance_output(json!({
            "gcode_flavor": flavor,
            "enable_pressure_advance": [true],
            "pressure_advance": "0.12"
        }))
        .await
        .unwrap();

        assert_eq!(pressure_advance_lines(&output), vec![expected]);
    }
}

#[tokio::test]
async fn pressure_advance_uses_first_numeric_value_from_supported_forms() {
    let cases = [
        (
            json!("0.07, 0.08"),
            "M900 K0.07; Override pressure advance value",
        ),
        (
            json!("0.09; 0.1"),
            "M900 K0.09; Override pressure advance value",
        ),
        (
            json!([0.11, 0.12]),
            "M900 K0.11; Override pressure advance value",
        ),
        (
            json!([0.13, "bad"]),
            "M900 K0.13; Override pressure advance value",
        ),
    ];

    for (value, expected) in cases {
        let output = pressure_advance_output(json!({
            "enable_pressure_advance": [true, false],
            "pressure_advance": value
        }))
        .await
        .unwrap();

        assert_eq!(pressure_advance_lines(&output), vec![expected]);
    }
}

#[tokio::test]
async fn pressure_advance_emits_after_custom_start_gcode() {
    let output = pressure_advance_output(json!({
        "machine_start_gcode": ";MACHINE-START",
        "filament_start_gcode": [";FILAMENT-START [filament_extruder_id]"],
        "enable_pressure_advance": [true],
        "pressure_advance": [0.031]
    }))
    .await
    .unwrap();

    assert_line_before(&output, ";MACHINE-START", ";FILAMENT-START 0");
    assert_line_before(
        &output,
        ";FILAMENT-START 0",
        "M900 K0.031; Override pressure advance value",
    );
    assert_line_before(
        &output,
        "M900 K0.031; Override pressure advance value",
        ";LAYER_CHANGE",
    );
}

#[tokio::test]
async fn pressure_advance_rejects_invalid_enabled_value() {
    let err = pressure_advance_output(json!({
        "enable_pressure_advance": ["bad", true],
        "pressure_advance": [0.03]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("enable_pressure_advance"));
}

#[tokio::test]
async fn pressure_advance_rejects_invalid_values() {
    for value in [json!(-0.01), json!(2.01), json!("bad"), json!([])] {
        let err = pressure_advance_output(json!({
            "enable_pressure_advance": true,
            "pressure_advance": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("pressure_advance"));
    }
}

async fn pressure_advance_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn pressure_advance_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| {
            line.starts_with("M900 K")
                || line.starts_with("SET_PRESSURE_ADVANCE")
                || line.starts_with("M572 D0 S")
                || line.starts_with("M233 X")
        })
        .collect()
}

fn assert_no_pressure_advance(output: &str) {
    assert!(pressure_advance_lines(output).is_empty());
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
