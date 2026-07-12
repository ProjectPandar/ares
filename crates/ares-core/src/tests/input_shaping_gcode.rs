use super::*;

#[tokio::test]
async fn default_and_disabled_machine_envelope_emit_no_input_shaping() {
    let default_output = slice_input_shaping_output(json!({})).await.unwrap();
    let disabled_envelope = slice_input_shaping_output(json!({
        "gcode_flavor": "marlin2",
        "emit_machine_limits_to_gcode": false,
        "input_shaping_emit": true,
        "input_shaping_freq_x": 40.0,
        "input_shaping_damp_x": 0.2
    }))
    .await
    .unwrap();

    assert_no_input_shaping(&default_output);
    assert_no_input_shaping(&disabled_envelope);
}

#[tokio::test]
async fn marlin_firmware_emits_input_shaping_after_machine_limits() {
    let output = slice_input_shaping_output(json!({
        "gcode_flavor": "marlin2",
        "input_shaping_emit": true,
        "input_shaping_type": "EI",
        "input_shaping_freq_x": 41.234,
        "input_shaping_freq_y": 52.346,
        "input_shaping_damp_x": 0.1234,
        "input_shaping_damp_y": 0.2345
    }))
    .await
    .unwrap();

    assert_eq!(
        input_shaping_lines(&output),
        vec![
            "M593 X F41.23 D0.123 ; Override input shaping",
            "M593 Y F52.35 D0.234 ; Override input shaping",
        ]
    );
    assert_line_before(
        &output,
        "M205 J0.010",
        "M593 X F41.23 D0.123 ; Override input shaping",
    );
    assert_line_before(
        &output,
        "M593 Y F52.35 D0.234 ; Override input shaping",
        "M190 S35 ; set bed temperature and wait for it to be reached",
    );
}

#[tokio::test]
async fn marlin_firmware_non_disable_zero_values_emit_axis_only_lines() {
    let output = slice_input_shaping_output(json!({
        "gcode_flavor": "marlin2",
        "input_shaping_emit": true,
        "input_shaping_type": "Default",
        "input_shaping_freq_x": 0.0,
        "input_shaping_freq_y": 0.0,
        "input_shaping_damp_x": 0.0,
        "input_shaping_damp_y": 0.0
    }))
    .await
    .unwrap();

    assert_eq!(
        input_shaping_lines(&output),
        vec![
            "M593 X ; Override input shaping",
            "M593 Y ; Override input shaping",
        ]
    );
}

#[tokio::test]
async fn reprap_firmware_emits_single_x_input_shaping_command() {
    let output = slice_input_shaping_output(json!({
        "gcode_flavor": "reprapfirmware",
        "input_shaping_emit": true,
        "input_shaping_type": "2HUMP_EI",
        "input_shaping_freq_x": 36.789,
        "input_shaping_freq_y": 99.0,
        "input_shaping_damp_x": 0.1119,
        "input_shaping_damp_y": 0.999
    }))
    .await
    .unwrap();

    assert_eq!(
        input_shaping_lines(&output),
        vec!["M593 P\"2HUMP_EI\" F36.79 S0.112 ; Override input shaping"]
    );
}

#[tokio::test]
async fn reprap_firmware_default_or_daa_zero_values_emit_no_command() {
    for input_type in ["Default", "DAA"] {
        let output = slice_input_shaping_output(json!({
            "gcode_flavor": "reprapfirmware",
            "input_shaping_emit": true,
            "input_shaping_type": input_type,
            "input_shaping_freq_x": 0.0,
            "input_shaping_freq_y": 80.0,
            "input_shaping_damp_x": 0.0,
            "input_shaping_damp_y": 0.9
        }))
        .await
        .unwrap();

        assert_no_input_shaping(&output);
    }
}

#[tokio::test]
async fn disable_type_emits_all_axis_zero_override() {
    let marlin = slice_input_shaping_output(json!({
        "gcode_flavor": "marlin2",
        "input_shaping_emit": true,
        "input_shaping_type": "Disable",
        "input_shaping_freq_x": 44.0,
        "input_shaping_freq_y": 55.0,
        "input_shaping_damp_x": 0.2,
        "input_shaping_damp_y": 0.3
    }))
    .await
    .unwrap();
    let reprap = slice_input_shaping_output(json!({
        "gcode_flavor": "reprapfirmware",
        "input_shaping_emit": true,
        "input_shaping_type": "Disable",
        "input_shaping_freq_x": 44.0,
        "input_shaping_damp_x": 0.2
    }))
    .await
    .unwrap();

    assert_eq!(
        input_shaping_lines(&marlin),
        vec!["M593 F0.00 D0.000 ; Override input shaping"]
    );
    assert_eq!(
        input_shaping_lines(&reprap),
        vec!["M593 F0.00 S0.000 ; Override input shaping"]
    );
}

#[tokio::test]
async fn unsupported_flavors_suppress_machine_envelope_input_shaping() {
    for flavor in ["marlin", "klipper", "repetier"] {
        let output = slice_input_shaping_output(json!({
            "gcode_flavor": flavor,
            "input_shaping_emit": true,
            "input_shaping_type": "MZV",
            "input_shaping_freq_x": 40.0,
            "input_shaping_damp_x": 0.2
        }))
        .await
        .unwrap();

        assert_no_input_shaping(&output);
    }
}

#[tokio::test]
async fn serialized_two_hump_type_names_are_accepted() {
    for input_type in ["2HUMP_EI", "3HUMP_EI"] {
        let output = slice_input_shaping_output(json!({
            "gcode_flavor": "reprapfirmware",
            "input_shaping_emit": true,
            "input_shaping_type": input_type,
            "input_shaping_freq_x": 32.0,
            "input_shaping_damp_x": 0.2
        }))
        .await
        .unwrap();

        assert_eq!(
            input_shaping_lines(&output),
            vec![format!(
                "M593 P\"{input_type}\" F32.00 S0.200 ; Override input shaping"
            )]
        );
    }
}

#[tokio::test]
async fn invalid_input_shaping_values_reject_slice_before_bytes() {
    let invalid_cases = [
        ("input_shaping_emit", json!("yes")),
        ("input_shaping_type", json!("TwoHumpEI")),
        ("input_shaping_type", json!("ThreeHumpEI")),
        ("input_shaping_type", json!("unknown")),
        ("input_shaping_freq_x", json!(-0.01)),
        ("input_shaping_freq_y", json!(1000.01)),
        ("input_shaping_damp_x", json!(-0.01)),
        ("input_shaping_damp_y", json!(1.01)),
        ("input_shaping_damp_y", json!("bad")),
    ];

    for (key, value) in invalid_cases {
        let err = slice_input_shaping_output(json!({
            "gcode_flavor": "marlin2",
            "input_shaping_emit": true,
            key: value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
}

async fn slice_input_shaping_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let output = slice(
        square_pyramid_ascii_stl(),
        merged_options(base_options(), extra),
    )
    .await?;
    Ok(String::from_utf8(output).unwrap())
}

fn input_shaping_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.starts_with("M593"))
        .map(str::to_owned)
        .collect()
}

fn assert_no_input_shaping(output: &str) {
    assert!(input_shaping_lines(output).is_empty());
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

fn base_options() -> serde_json::Value {
    json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0
    })
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
