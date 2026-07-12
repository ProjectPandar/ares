use super::*;

#[tokio::test]
async fn marlin2_enable_emits_second_layer_enable_and_final_disable() {
    let output = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "enable"
    }))
    .await
    .unwrap();

    assert_eq!(power_loss_lines(&output), vec!["M413 S1", "M413 S0"]);
    assert_after_z_before_segment_count(&output, 1, "M413 S1");
    assert_line_before(&output, "M413 S0", "M73 P100 R0");
    assert_line_before(&output, "M413 S0", "M2");
}

#[tokio::test]
async fn marlin2_disable_emits_second_layer_disable_only() {
    let output = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "disable"
    }))
    .await
    .unwrap();

    assert_eq!(power_loss_lines(&output), vec!["M413 S0"]);
    assert_after_z_before_segment_count(&output, 1, "M413 S0");
}

#[tokio::test]
async fn printer_configuration_omits_power_loss_recovery_gcode() {
    let default_output = power_loss_output(json!({
        "gcode_flavor": "marlin2"
    }))
    .await
    .unwrap();
    let explicit_output = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "printer_configuration"
    }))
    .await
    .unwrap();

    assert!(power_loss_lines(&default_output).is_empty());
    assert!(power_loss_lines(&explicit_output).is_empty());
}

#[tokio::test]
async fn unsupported_flavors_omit_power_loss_recovery_gcode() {
    for flavor in ["marlin", "klipper", "reprapfirmware", "repetier"] {
        let output = power_loss_output(json!({
            "gcode_flavor": flavor,
            "enable_power_loss_recovery": "enable"
        }))
        .await
        .unwrap();

        assert!(
            power_loss_lines(&output).is_empty(),
            "expected no power loss recovery G-code for {flavor}"
        );
    }
}

#[tokio::test]
async fn power_loss_recovery_comment_follows_gcode_comments() {
    let output = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "enable",
        "gcode_comments": true
    }))
    .await
    .unwrap();

    assert_eq!(
        power_loss_lines(&output),
        vec![
            "M413 S1 ; set Power-loss Recovery",
            "M413 S0 ; set Power-loss Recovery"
        ]
    );
}

#[tokio::test]
async fn invalid_power_loss_recovery_mode_reaches_slice_error() {
    let err = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "maybe"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("enable_power_loss_recovery"));
}

#[tokio::test]
async fn non_string_power_loss_recovery_mode_reaches_slice_error() {
    let err = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": true
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("enable_power_loss_recovery"));
}

#[tokio::test]
async fn single_layer_print_omits_power_loss_recovery_gcode() {
    let output = power_loss_output(json!({
        "gcode_flavor": "marlin2",
        "enable_power_loss_recovery": "enable",
        "layer_height": 0.4,
        "initial_layer_print_height": 0.4
    }))
    .await
    .unwrap();

    assert!(output.lines().any(|line| line == "; layer_count = 1"));
    assert!(power_loss_lines(&output).is_empty());
}

async fn power_loss_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn power_loss_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M413 "))
        .collect()
}

fn assert_after_z_before_segment_count(output: &str, layer: usize, expected: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let layer_index = lines
        .iter()
        .position(|line| *line == format!(";LAYER:{layer}"))
        .unwrap();
    let z_index = lines[layer_index..]
        .iter()
        .position(|line| line.starts_with("G1 Z"))
        .map(|index| layer_index + index)
        .unwrap();
    let expected_index = lines[z_index..]
        .iter()
        .position(|line| *line == expected)
        .map(|index| z_index + index)
        .unwrap();
    let segment_count_index = lines[z_index..]
        .iter()
        .position(|line| line.starts_with("; segment_count = "))
        .map(|index| z_index + index)
        .unwrap();

    assert!(z_index < expected_index, "{z_index} !< {expected_index}");
    assert!(
        expected_index < segment_count_index,
        "{expected_index} !< {segment_count_index}"
    );
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
