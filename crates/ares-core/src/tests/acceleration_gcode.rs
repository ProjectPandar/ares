use super::*;

mod bridge;

#[tokio::test]
async fn default_accelerations_emit_before_relevant_moves() {
    let output = slice_acceleration_output(json!({})).await;

    assert_acceleration_before_move(
        &output,
        "M204 S10000",
        ";MOVE:travel:external_perimeter:-0.5,0",
    );
    assert_acceleration_before_move(
        &output,
        "M204 S300",
        ";MOVE:print:external_perimeter:0,-0.5",
    );
    assert_acceleration_before_move(&output, "M204 S500", ";MOVE:print:sparse_infill:-0.75,0.25");
    assert_eq!(movement_command_count(&output), 27);
}

#[tokio::test]
async fn default_acceleration_zero_disables_acceleration_commands() {
    let output = slice_acceleration_output(json!({
        "default_acceleration": 0,
        "z_hop": 0
    }))
    .await;

    assert_eq!(acceleration_lines(&output), Vec::<&str>::new());
    assert_eq!(movement_command_count(&output), 27);
    assert!(output.lines().any(|line| line == "G1 X-0.5 Y0 F7200"));
    assert!(output.lines().any(|line| line == "G1 X-0.5 Y0 E0.02393"));
}

#[test]
fn initial_layer_acceleration_overrides_first_layer_print_roles() {
    let output = rectangle_acceleration_output(json!({
        "default_acceleration": 700,
        "initial_layer_acceleration": 250,
        "outer_wall_acceleration": 450,
        "inner_wall_acceleration": 650,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }));

    assert_acceleration_before_move(&output, "M204 S900", ";MOVE:travel:external_perimeter:0,0");
    assert_acceleration_before_move(&output, "M204 S250", ";MOVE:print:external_perimeter:4,0");
    assert_acceleration_before_move(
        &output,
        "M204 S250",
        ";MOVE:print:internal_perimeter:3.24292,0.75708",
    );
    assert_acceleration_before_move(&output, "M204 S250", ";MOVE:print:sparse_infill:1.2,3.1");
    assert!(!output.contains("M204 S450"));
    assert!(!output.contains("M204 S650"));
    assert!(!output.contains("M204 S350"));
}

#[test]
fn role_accelerations_apply_when_initial_layer_acceleration_is_zero() {
    let output = rectangle_acceleration_output(json!({
        "default_acceleration": 700,
        "initial_layer_acceleration": 0,
        "outer_wall_acceleration": 450,
        "inner_wall_acceleration": 650,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }));

    assert_acceleration_before_move(&output, "M204 S900", ";MOVE:travel:external_perimeter:0,0");
    assert_acceleration_before_move(&output, "M204 S450", ";MOVE:print:external_perimeter:4,0");
    assert_acceleration_before_move(
        &output,
        "M204 S650",
        ";MOVE:print:internal_perimeter:3.24292,0.75708",
    );
    assert_acceleration_before_move(&output, "M204 S350", ";MOVE:print:sparse_infill:1.2,3.1");
}

#[tokio::test]
async fn non_first_layer_role_accelerations_apply_by_role() {
    let output = slice_acceleration_output(json!({
        "default_acceleration": 700,
        "initial_layer_acceleration": 250,
        "outer_wall_acceleration": 450,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "M204 S900",
        ";MOVE:travel:sparse_infill:-0.75,-0.25",
    );
    assert_acceleration_before_move(&output, "M204 S350", ";MOVE:print:sparse_infill:-0.75,0.25");
    assert_acceleration_before_move(&output, "M204 S450", ";MOVE:print:external_perimeter:0,-1");
}

#[tokio::test]
async fn numeric_sparse_infill_acceleration_and_rounding_are_used() {
    let output = slice_acceleration_output(json!({
        "sparse_infill_acceleration": 333.6,
        "travel_acceleration": 1000.4
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "M204 S1000",
        ";MOVE:travel:external_perimeter:-0.5,0",
    );
    assert_acceleration_before_move(&output, "M204 S334", ";MOVE:print:sparse_infill:-0.75,0.25");
}

#[tokio::test]
async fn klipper_acceleration_emits_accel_to_decel_from_existing_options() {
    let output = slice_acceleration_output(json!({
        "gcode_flavor": "klipper",
        "sparse_infill_acceleration": 333.6,
        "accel_to_decel_factor": 33
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "SET_VELOCITY_LIMIT ACCEL=334 ACCEL_TO_DECEL=110",
        ";MOVE:print:sparse_infill:-0.75,0.25",
    );
    assert!(!output.lines().any(|line| line.starts_with("M204 S")));
}

#[tokio::test]
async fn klipper_acceleration_truncates_after_applying_decimal_factor() {
    let output = slice_acceleration_output(json!({
        "gcode_flavor": "klipper",
        "sparse_infill_acceleration": 333.6,
        "accel_to_decel_factor": 33.5
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "SET_VELOCITY_LIMIT ACCEL=334 ACCEL_TO_DECEL=111",
        ";MOVE:print:sparse_infill:-0.75,0.25",
    );
}

#[tokio::test]
async fn klipper_acceleration_omits_accel_to_decel_when_option_is_disabled() {
    let output = slice_acceleration_output(json!({
        "gcode_flavor": "klipper",
        "default_acceleration": 500,
        "initial_layer_acceleration": 500,
        "outer_wall_acceleration": 500,
        "travel_acceleration": 500,
        "sparse_infill_acceleration": "100%",
        "accel_to_decel_enable": false
    }))
    .await;

    assert_eq!(
        acceleration_lines(&output),
        vec!["SET_VELOCITY_LIMIT ACCEL=500"]
    );
    assert!(!output.contains("ACCEL_TO_DECEL"));
}

#[tokio::test]
async fn travel_acceleration_zero_does_not_fallback_to_default() {
    let output = slice_acceleration_output(json!({
        "default_acceleration": 700,
        "travel_acceleration": 0
    }))
    .await;

    assert_no_acceleration_before_move(&output, ";MOVE:travel:external_perimeter:-0.5,0");
    assert_acceleration_before_move(&output, "M204 S300", ";MOVE:print:skirt:2.5,-2.5");
}

#[tokio::test]
async fn acceleration_commands_are_suppressed_when_unchanged() {
    let output = slice_acceleration_output(json!({
        "default_acceleration": 500,
        "initial_layer_acceleration": 500,
        "outer_wall_acceleration": 500,
        "travel_acceleration": 500,
        "sparse_infill_acceleration": "100%",
        "z_hop": 0
    }))
    .await;

    assert_eq!(acceleration_lines(&output), vec!["M204 S500"]);
    assert_eq!(movement_command_count(&output), 27);
}

#[tokio::test]
async fn invalid_acceleration_values_are_rejected() {
    for (key, value) in [
        ("default_acceleration", json!(-1)),
        ("initial_layer_acceleration", json!("bad")),
        ("outer_wall_acceleration", json!("inf")),
        ("inner_wall_acceleration", json!(true)),
        ("travel_acceleration", json!(-0.1)),
        ("bridge_acceleration", json!("bad%")),
        ("bridge_acceleration", json!(-1)),
        ("bridge_acceleration", json!(false)),
        ("sparse_infill_acceleration", json!("bad%")),
        ("sparse_infill_acceleration", json!(-1)),
    ] {
        let err = slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({ key: value })).unwrap(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)), "{key}");
    }
}

#[tokio::test]
async fn gcode_comments_apply_to_acceleration_commands() {
    let output = slice_acceleration_output(json!({
        "gcode_comments": true,
        "default_acceleration": 500,
        "initial_layer_acceleration": 500,
        "outer_wall_acceleration": 500,
        "travel_acceleration": 500,
        "sparse_infill_acceleration": "100%",
        "z_hop": 0
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "M204 S500 ; adjust acceleration")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 E0.02393 ; extrude")
    );
}

async fn slice_acceleration_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "seam_gap": 0,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.25,
            "minimum_sparse_infill_area": 0,
            "infill_direction": 0,
            "sparse_infill_pattern": "alignedrectilinear",
            "is_infill_first": true,
            "infill_anchor_max": 0,
            "bottom_shell_layers": 0,
            "top_shell_layers": 0
        }),
        extra,
    );
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    String::from_utf8(output).unwrap()
}

fn rectangle_acceleration_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "wall_loops": 3,
            "line_width": 0.4,
            "seam_gap": 0,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.4,
            "minimum_sparse_infill_area": 0,
            "infill_direction": 0,
            "brim_width": 0,
            "skirt_loops": 0,
            "infill_anchor_max": 0,
            "bottom_shell_layers": 0,
            "top_shell_layers": 0
        }),
        extra,
    );
    String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::rectangular_pipeline(&options),
            &options,
        )
        .unwrap(),
    )
    .unwrap()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

fn acceleration_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M204 S") || line.starts_with("SET_VELOCITY_LIMIT ACCEL="))
        .collect()
}

fn movement_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}

fn assert_acceleration_before_move(output: &str, acceleration: &str, marker: &str) {
    assert_eq!(
        first_non_comment_after_marker(output, marker),
        Some(acceleration)
    );
}

fn assert_no_acceleration_before_move(output: &str, marker: &str) {
    assert!(
        !first_non_comment_after_marker(output, marker).is_some_and(|line| {
            line.starts_with("M204 ") || line.starts_with("SET_VELOCITY_LIMIT ACCEL=")
        })
    );
}

fn first_non_comment_after_marker<'a>(output: &'a str, marker: &str) -> Option<&'a str> {
    let lines = output.lines().collect::<Vec<_>>();
    let marker_index = lines.iter().position(|line| *line == marker).unwrap();
    lines[marker_index + 1..]
        .iter()
        .copied()
        .find(|line| !line.starts_with(';'))
}
