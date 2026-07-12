use super::*;

#[tokio::test]
async fn bambu_scan_first_layer_emits_second_layer_inspection_block() {
    let output = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": true
    }))
    .await
    .unwrap();

    assert_eq!(
        scan_lines(&output),
        vec![
            "M976 S1 P1 ; scan model before printing 2nd layer",
            "M400 P100"
        ]
    );
    assert_after_z_before_segment_count(
        &output,
        1,
        "M976 S1 P1 ; scan model before printing 2nd layer",
    );
    assert_line_before(
        &output,
        "M976 S1 P1 ; scan model before printing 2nd layer",
        "M400 P100",
    );
}

#[tokio::test]
async fn non_bambu_printer_omits_scan_first_layer_gcode() {
    let output = scan_first_layer_output(json!({
        "printer_model": "Prusa XL",
        "scan_first_layer": true
    }))
    .await
    .unwrap();

    assert!(scan_lines(&output).is_empty());
}

#[tokio::test]
async fn missing_scan_first_layer_defaults_to_no_inspection_block() {
    let output = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon"
    }))
    .await
    .unwrap();

    assert!(scan_lines(&output).is_empty());
}

#[tokio::test]
async fn missing_printer_model_omits_scan_first_layer_gcode() {
    let output = scan_first_layer_output(json!({
        "scan_first_layer": true
    }))
    .await
    .unwrap();

    assert!(scan_lines(&output).is_empty());
}

#[tokio::test]
async fn disabled_scan_first_layer_omits_bambu_inspection_block() {
    let output = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": false
    }))
    .await
    .unwrap();

    assert!(scan_lines(&output).is_empty());
}

#[tokio::test]
async fn invalid_scan_first_layer_reaches_slice_error() {
    let err = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("scan_first_layer"));
}

#[tokio::test]
async fn invalid_scan_first_layer_reaches_slice_error_on_single_layer_print() {
    let err = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": "true",
        "layer_height": 0.4,
        "initial_layer_print_height": 0.4
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("scan_first_layer"));
}

#[tokio::test]
async fn single_layer_print_omits_scan_first_layer_gcode() {
    let output = scan_first_layer_output(json!({
        "printer_model": "Bambu Lab X1 Carbon",
        "scan_first_layer": true,
        "layer_height": 0.4,
        "initial_layer_print_height": 0.4
    }))
    .await
    .unwrap();

    assert!(output.lines().any(|line| line == "; layer_count = 1"));
    assert!(scan_lines(&output).is_empty());
}

async fn scan_first_layer_output(extra: serde_json::Value) -> Result<String, SliceError> {
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

fn scan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M976 ") || line.starts_with("M400 P100"))
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
