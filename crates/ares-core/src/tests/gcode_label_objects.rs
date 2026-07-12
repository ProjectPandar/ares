use super::*;
use serde_json::{Value, json};

#[tokio::test]
async fn default_slice_labels_single_object_print_moves() {
    let output = label_output(json!({})).await.unwrap();

    assert_eq!(
        count_lines(&output, "; printing object ares-object-0 id:0 copy 0"),
        1
    );
    assert_eq!(
        count_lines(&output, "; stop printing object ares-object-0 id:0 copy 0"),
        1
    );
    assert_object_label_order(&output);
    assert_no_exclusion_commands(&output);
}

#[tokio::test]
async fn gcode_label_objects_false_suppresses_object_labels_only() {
    let output = label_output(json!({
        "gcode_label_objects": false
    }))
    .await
    .unwrap();

    assert!(!output.contains("; printing object "));
    assert!(!output.contains("; stop printing object "));
    assert!(output.contains(";MOVE:"));
}

#[tokio::test]
async fn klipper_exclude_object_emits_definition_and_move_markers() {
    let output = label_output(json!({
        "gcode_flavor": "klipper",
        "exclude_object": true,
        "machine_start_gcode": ";MACHINE-START"
    }))
    .await
    .unwrap();

    assert_eq!(count_lines(&output, KLIPPER_DEFAULT_DEFINITION), 1);
    assert_eq!(count_lines(&output, KLIPPER_START), 1);
    assert_eq!(count_lines(&output, KLIPPER_END), 1);
    assert_order(&output, ";MACHINE-START", KLIPPER_DEFAULT_DEFINITION);
    assert_order(&output, KLIPPER_DEFAULT_DEFINITION, "M73 P0 R0");
    assert_order(&output, KLIPPER_START, first_move_line(&output));
    assert_order(&output, last_move_line(&output), KLIPPER_END);
    assert_object_label_order(&output);
}

#[tokio::test]
async fn klipper_exclude_object_definition_uses_first_layer_bounds_without_skirt() {
    let output = label_output(json!({
        "gcode_flavor": "klipper",
        "exclude_object": true,
        "skirt_loops": 0
    }))
    .await
    .unwrap();

    assert_eq!(count_lines(&output, KLIPPER_NO_SKIRT_DEFINITION), 1);
    assert_eq!(count_lines(&output, KLIPPER_DEFAULT_DEFINITION), 0);
    assert_eq!(count_lines(&output, KLIPPER_START), 1);
    assert_eq!(count_lines(&output, KLIPPER_END), 1);
    assert_order(&output, KLIPPER_NO_SKIRT_DEFINITION, "M73 P0 R0");
}

#[tokio::test]
async fn marlin_exclude_object_emits_m486_definition_and_move_markers() {
    assert_marlin_m486_output("marlin").await;
}

#[tokio::test]
async fn marlin2_exclude_object_emits_m486_definition_and_move_markers() {
    assert_marlin_m486_output("marlin2").await;
}

#[tokio::test]
async fn reprap_firmware_exclude_object_emits_named_m486_definition() {
    let output = label_output(json!({
        "gcode_flavor": "reprapfirmware",
        "exclude_object": true
    }))
    .await
    .unwrap();

    assert_eq!(count_lines(&output, "M486 S0 A\"ares-object-0\""), 1);
    assert_eq!(line_indices(&output, "M486 S0").len(), 1);
    assert_eq!(line_indices(&output, "M486 S-1").len(), 2);

    let definition_index = line_index(&output, "M486 S0 A\"ares-object-0\"");
    let stop_indices = line_indices(&output, "M486 S-1");
    let start_index = line_index(&output, "M486 S0");
    let first_move_index = first_move_index(&output);
    let last_move_index = last_move_index(&output);
    assert!(definition_index < stop_indices[0]);
    assert!(stop_indices[0] < start_index);
    assert!(start_index < first_move_index);
    assert!(last_move_index < stop_indices[1]);
}

#[tokio::test]
async fn disabled_labels_still_emit_klipper_exclusion_commands() {
    let output = label_output(json!({
        "gcode_flavor": "klipper",
        "gcode_label_objects": false,
        "exclude_object": true
    }))
    .await
    .unwrap();

    assert!(!output.contains("; printing object "));
    assert!(!output.contains("; stop printing object "));
    assert_eq!(count_lines(&output, KLIPPER_DEFAULT_DEFINITION), 1);
    assert_eq!(count_lines(&output, KLIPPER_START), 1);
    assert_eq!(count_lines(&output, KLIPPER_END), 1);
    assert_order(&output, KLIPPER_START, first_move_line(&output));
    assert_order(&output, last_move_line(&output), KLIPPER_END);
}

#[tokio::test]
async fn repetier_exclude_object_is_noop_but_keeps_labels() {
    let output = label_output(json!({
        "gcode_flavor": "repetier",
        "exclude_object": true
    }))
    .await
    .unwrap();

    assert_object_label_order(&output);
    assert_no_exclusion_commands(&output);
}

#[tokio::test]
async fn gcode_label_objects_rejects_non_boolean_values() {
    let err = label_output(json!({
        "gcode_label_objects": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("gcode_label_objects must be a boolean")
    );
}

#[tokio::test]
async fn exclude_object_rejects_non_boolean_values() {
    let err = label_output(json!({
        "exclude_object": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("exclude_object must be a boolean"));
}

#[tokio::test]
async fn support_object_skip_flush_rejects_invalid_values_during_gcode_formatting() {
    for value in [json!("true"), json!("false"), json!(1), Value::Null] {
        let err = label_output(json!({
            "support_object_skip_flush": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_object_skip_flush"));
    }
}

#[tokio::test]
async fn support_object_skip_flush_true_and_false_preserve_klipper_exclude_object_output() {
    let disabled = label_output(json!({
        "gcode_flavor": "klipper",
        "exclude_object": true,
        "support_object_skip_flush": false
    }))
    .await
    .unwrap();
    let enabled = label_output(json!({
        "gcode_flavor": "klipper",
        "exclude_object": true,
        "support_object_skip_flush": true
    }))
    .await
    .unwrap();

    assert_eq!(disabled, enabled);
}

const KLIPPER_DEFAULT_DEFINITION: &str = "EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0 POLYGON=[[-2.5,-2.5],[2.5,-2.5],[2.5,2.5],[-2.5,2.5],[-2.5,-2.5]]";
const KLIPPER_NO_SKIRT_DEFINITION: &str = "EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER=0,0 POLYGON=[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5],[-0.5,-0.5]]";
const KLIPPER_START: &str = "EXCLUDE_OBJECT_START NAME=ares-object-0";
const KLIPPER_END: &str = "EXCLUDE_OBJECT_END NAME=ares-object-0";

async fn assert_marlin_m486_output(gcode_flavor: &str) {
    let output = label_output(json!({
        "gcode_flavor": gcode_flavor,
        "exclude_object": true
    }))
    .await
    .unwrap();

    assert_eq!(line_indices(&output, "M486 S0").len(), 2);
    assert_eq!(count_lines(&output, "M486 Aares-object-0"), 1);
    assert_eq!(line_indices(&output, "M486 S-1").len(), 2);

    let start_indices = line_indices(&output, "M486 S0");
    let stop_indices = line_indices(&output, "M486 S-1");
    let name_index = line_index(&output, "M486 Aares-object-0");
    let first_move_index = first_move_index(&output);
    let last_move_index = last_move_index(&output);

    assert!(start_indices[0] < name_index);
    assert!(name_index < stop_indices[0]);
    assert!(stop_indices[0] < start_indices[1]);
    assert!(start_indices[1] < first_move_index);
    assert!(last_move_index < stop_indices[1]);
}

async fn label_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options: SliceOptions = serde_json::from_value(extra).unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await?;
    Ok(String::from_utf8(output).unwrap())
}

fn count_lines(output: &str, expected: &str) -> usize {
    output.lines().filter(|line| *line == expected).count()
}

fn line_index(output: &str, expected: &str) -> usize {
    output
        .lines()
        .position(|line| line == expected)
        .unwrap_or_else(|| panic!("missing line: {expected}"))
}

fn line_indices(output: &str, expected: &str) -> Vec<usize> {
    output
        .lines()
        .enumerate()
        .filter_map(|(index, line)| (line == expected).then_some(index))
        .collect()
}

fn assert_order(output: &str, before: &str, after: &str) {
    assert!(line_index(output, before) < line_index(output, after));
}

fn first_move_line(output: &str) -> &str {
    output
        .lines()
        .find(|line| line.starts_with(";MOVE:"))
        .expect("missing first move marker")
}

fn last_move_line(output: &str) -> &str {
    output
        .lines()
        .rfind(|line| line.starts_with(";MOVE:"))
        .expect("missing last move marker")
}

fn first_move_index(output: &str) -> usize {
    output
        .lines()
        .position(|line| line.starts_with(";MOVE:"))
        .expect("missing first move marker")
}

fn last_move_index(output: &str) -> usize {
    output
        .lines()
        .collect::<Vec<_>>()
        .iter()
        .rposition(|line| line.starts_with(";MOVE:"))
        .expect("missing last move marker")
}

fn assert_no_exclusion_commands(output: &str) {
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("EXCLUDE_OBJECT_DEFINE")),
        "unexpected EXCLUDE_OBJECT_DEFINE in output"
    );
    for line in [
        KLIPPER_START,
        KLIPPER_END,
        "M486 S0",
        "M486 Aares-object-0",
        "M486 S-1",
        "M486 S0 A\"ares-object-0\"",
    ] {
        assert_eq!(count_lines(output, line), 0);
    }
}

fn assert_object_label_order(output: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let start_index = lines
        .iter()
        .position(|line| *line == "; printing object ares-object-0 id:0 copy 0")
        .expect("missing object start label");
    let first_move_index = lines
        .iter()
        .position(|line| line.starts_with(";MOVE:"))
        .expect("missing first move marker");
    let last_move_index = lines
        .iter()
        .rposition(|line| line.starts_with(";MOVE:"))
        .expect("missing last move marker");
    let stop_index = lines
        .iter()
        .position(|line| *line == "; stop printing object ares-object-0 id:0 copy 0")
        .expect("missing object stop label");
    let finish_index = lines
        .iter()
        .position(|line| *line == "M73 P100 R0")
        .expect("missing final progress marker");

    assert!(start_index < first_move_index);
    assert!(last_move_index < stop_index);
    assert!(stop_index < finish_index);
}
