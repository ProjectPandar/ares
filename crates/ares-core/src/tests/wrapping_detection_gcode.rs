use super::*;

#[tokio::test]
async fn enabled_wrapping_detection_emits_rendered_block_once_per_layer() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num] [layer_z] [max_layer_z]"
    }))
    .await
    .unwrap();

    assert_eq!(
        wrap_lines(&output),
        vec![";WRAP 1 0.2 0.2", ";WRAP 2 0.4 0.4"]
    );
    assert_line_after(&output, ";WRAP 1 0.2 0.2", "G1 Z0.2 F7200");
    assert_line_after(&output, ";WRAP 2 0.4 0.4", "G1 Z0.4 F7200");
}

#[tokio::test]
async fn wrapping_detection_requires_active_wrapping_exclude_area() {
    for inactive in [
        None,
        Some(json!("")),
        Some(json!("0x0")),
        Some(json!([])),
        Some(json!("0x0,10x0")),
        Some(json!([[0.0, 0.0], [10.0, 0.0]])),
    ] {
        let mut extra = json!({
            "enable_wrapping_detection": true,
            "wrapping_detection_gcode": ";WRAP [layer_num]"
        });
        if let Some(value) = inactive {
            extra["wrapping_exclude_area"] = value;
        }

        let output = wrapping_output(extra).await.unwrap();

        assert!(
            wrap_lines(&output).is_empty(),
            "inactive wrapping_exclude_area should suppress wrapping detection in:\n{output}"
        );
    }
}

#[tokio::test]
async fn active_wrapping_exclude_area_keeps_wrapping_detection_output() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num] [layer_z]"
    }))
    .await
    .unwrap();

    assert_eq!(wrap_lines(&output), vec![";WRAP 1 0.2", ";WRAP 2 0.4"]);
}

#[tokio::test]
async fn json_wrapping_exclude_area_keeps_wrapping_detection_output() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]],
        "wrapping_detection_gcode": ";WRAP [layer_num]"
    }))
    .await
    .unwrap();

    assert_eq!(wrap_lines(&output), vec![";WRAP 1", ";WRAP 2"]);
}

#[tokio::test]
async fn wrapping_detection_layers_limits_rendered_blocks() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_detection_layers": 1,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num] [layer_z]"
    }))
    .await
    .unwrap();

    assert_eq!(wrap_lines(&output), vec![";WRAP 1 0.2"]);
}

#[tokio::test]
async fn wrapping_detection_layers_zero_suppresses_rendered_blocks() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_detection_layers": 0,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num] [layer_z]"
    }))
    .await
    .unwrap();

    assert!(wrap_lines(&output).is_empty());
}

#[tokio::test]
async fn wrapping_detection_layers_accepts_integer_string() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_detection_layers": "1",
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num] [layer_z]"
    }))
    .await
    .unwrap();

    assert_eq!(wrap_lines(&output), vec![";WRAP 1 0.2"]);
}

#[tokio::test]
async fn omitted_wrapping_detection_layers_uses_orca_default_window() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num]"
    }))
    .await
    .unwrap();

    assert_eq!(wrap_lines(&output), vec![";WRAP 1", ";WRAP 2"]);
}

#[tokio::test]
async fn wrapping_detection_emits_after_power_loss_before_scan_and_segment_output() {
    let output = wrapping_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": true,
        "enable_wrapping_detection": true,
        "enable_power_loss_recovery": "enable",
        "gcode_flavor": "marlin2",
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP [layer_num]"
    }))
    .await
    .unwrap();

    assert_line_before(&output, "M413 S1", ";WRAP 2");
    assert_line_before(
        &output,
        ";WRAP 2",
        "M976 S1 P1 ; scan model before printing 2nd layer",
    );
    assert_line_after_anchor_before(
        &output,
        ";LAYER:1",
        "M976 S1 P1 ; scan model before printing 2nd layer",
        "; segment_count = 4",
    );
}

#[tokio::test]
async fn disabled_missing_or_empty_wrapping_detection_is_noop() {
    let absent = wrapping_output(json!({})).await.unwrap();
    let disabled = wrapping_output(json!({
        "enable_wrapping_detection": false,
        "wrapping_detection_gcode": ";WRAP [layer_num]"
    }))
    .await
    .unwrap();
    let empty = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_detection_gcode": ""
    }))
    .await
    .unwrap();

    assert!(wrap_lines(&absent).is_empty());
    assert_eq!(
        without_option_count(&absent),
        without_option_count(&disabled)
    );
    assert_eq!(without_option_count(&absent), without_option_count(&empty));
}

#[tokio::test]
async fn wrapping_detection_replaces_brace_and_physical_extruder_placeholders() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": ";WRAP {layer_num} {layer_z} {max_layer_z} {most_used_physical_extruder_id} {curr_physical_extruder_id}"
    }))
    .await
    .unwrap();

    assert_eq!(
        wrap_lines(&output),
        vec![";WRAP 1 0.2 0.2 0 0", ";WRAP 2 0.4 0.4 0 0"]
    );
}

#[tokio::test]
async fn wrapping_detection_preserves_unknown_conditionals_and_expressions() {
    let output = wrapping_output(json!({
        "enable_wrapping_detection": true,
        "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
        "wrapping_detection_gcode": "{if layer_num == 3}\n;WRAP {layer_num+1} [future] [layer_num]\n{endif}"
    }))
    .await
    .unwrap();

    assert_line_after(&output, "{if layer_num == 3}", "G1 Z0.2 F7200");
    assert_line_after(
        &output,
        ";WRAP {layer_num+1} [future] 1",
        "{if layer_num == 3}",
    );
    assert_line_after(&output, "{endif}", ";WRAP {layer_num+1} [future] 1");
}

#[tokio::test]
async fn invalid_enable_wrapping_detection_reaches_slice_error() {
    let err = wrapping_output(json!({
        "enable_wrapping_detection": "true",
        "wrapping_detection_gcode": ";WRAP [layer_num]"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("enable_wrapping_detection"));
}

#[tokio::test]
async fn invalid_wrapping_detection_layers_reaches_slice_error() {
    for invalid in [
        json!(-1),
        json!(1.5),
        json!(true),
        json!(null),
        json!(["1"]),
        json!({"layers": 1}),
        json!("bad"),
    ] {
        let err = wrapping_output(json!({
            "enable_wrapping_detection": true,
            "wrapping_detection_layers": invalid,
            "wrapping_exclude_area": "0x0,10x0,10x10,0x10",
            "wrapping_detection_gcode": ";WRAP [layer_num]"
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("wrapping_detection_layers"));
    }
}

#[tokio::test]
async fn invalid_wrapping_exclude_area_reaches_slice_error() {
    for invalid in [
        json!("0x0,"),
        json!("0x0,bad,10x10"),
        json!("0x0,10x0x1,10x10"),
        json!("0x0,10x0,NaNx10"),
        json!("0x0,10x0,infx10"),
        json!([[0.0], [10.0, 0.0], [10.0, 10.0]]),
        json!({"x": 0.0, "y": 0.0}),
    ] {
        let err = wrapping_output(json!({
            "enable_wrapping_detection": true,
            "wrapping_exclude_area": invalid,
            "wrapping_detection_gcode": ";WRAP [layer_num]"
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("wrapping_exclude_area"));
    }
}

#[tokio::test]
async fn invalid_wrapping_exclude_area_reaches_slice_error_even_when_inactive() {
    for extra in [
        json!({
            "enable_wrapping_detection": false,
            "wrapping_exclude_area": "0x0,bad,10x10",
            "wrapping_detection_gcode": ";WRAP [layer_num]"
        }),
        json!({
            "enable_wrapping_detection": true,
            "wrapping_exclude_area": "0x0,bad,10x10",
            "wrapping_detection_gcode": ""
        }),
    ] {
        let err = wrapping_output(extra).await.unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("wrapping_exclude_area"));
    }
}

#[tokio::test]
async fn invalid_wrapping_detection_gcode_reaches_slice_error_even_when_disabled() {
    let err = wrapping_output(json!({
        "enable_wrapping_detection": false,
        "wrapping_detection_gcode": ["; invalid"]
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("wrapping_detection_gcode"));
}

async fn wrapping_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn wrap_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with(";WRAP "))
        .collect()
}

fn without_option_count(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("; option_count = "))
        .collect::<Vec<_>>()
        .join("\n")
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

fn assert_line_after(output: &str, first: &str, second: &str) {
    assert_line_before(output, second, first);
}

fn assert_line_after_anchor_before(output: &str, anchor: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let anchor_index = lines.iter().position(|line| *line == anchor).unwrap();
    let first_index = lines[anchor_index..]
        .iter()
        .position(|line| *line == first)
        .map(|index| anchor_index + index)
        .unwrap();
    let second_index = lines[anchor_index..]
        .iter()
        .position(|line| *line == second)
        .map(|index| anchor_index + index)
        .unwrap();
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
