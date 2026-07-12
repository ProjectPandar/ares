use super::*;

#[tokio::test]
async fn default_jerk_zero_suppresses_jerk_commands() {
    let output = slice_jerk_output(json!({ "z_hop": 0 })).await;

    assert_eq!(jerk_lines(&output), Vec::<&str>::new());
    assert_eq!(movement_command_count(&output), 27);
    assert!(output.lines().any(|line| line == "G1 X-0.5 Y0 F7200"));
}

#[tokio::test]
async fn jerk_commands_emit_before_relevant_moves() {
    let output = slice_jerk_output(json!({
        "default_jerk": 8,
        "initial_layer_jerk": 6,
        "travel_jerk": 11,
        "initial_layer_travel_jerk": "50%",
        "outer_wall_jerk": 7,
        "infill_jerk": 5
    }))
    .await;

    assert_jerk_before_move(
        &output,
        "M205 X5.5 Y5.5",
        ";MOVE:travel:external_perimeter:-0.5,0",
    );
    assert_jerk_before_move(&output, "M205 X6 Y6", ";MOVE:print:skirt:2.5,-2.5");
    assert_jerk_before_move(
        &output,
        "M205 X5 Y5",
        ";MOVE:print:sparse_infill:-0.75,0.25",
    );
    assert_jerk_before_move(
        &output,
        "M205 X11 Y11",
        ";MOVE:travel:sparse_infill:-0.75,-0.25",
    );
    assert_jerk_before_move(&output, "M205 X7 Y7", ";MOVE:print:external_perimeter:0,-1");
}

#[test]
fn role_jerks_apply_when_initial_layer_jerk_is_zero() {
    let output = rectangle_jerk_output(json!({
        "default_jerk": 8,
        "initial_layer_jerk": 0,
        "outer_wall_jerk": 7,
        "inner_wall_jerk": 4,
        "infill_jerk": 5,
        "travel_jerk": 11
    }));

    assert_jerk_before_move(
        &output,
        "M205 X11 Y11",
        ";MOVE:travel:external_perimeter:0,0",
    );
    assert_jerk_before_move(&output, "M205 X7 Y7", ";MOVE:print:external_perimeter:4,0");
    assert_jerk_before_move(
        &output,
        "M205 X4 Y4",
        ";MOVE:print:internal_perimeter:3.24292,0.75708",
    );
    assert_jerk_before_move(&output, "M205 X5 Y5", ";MOVE:print:sparse_infill:1.2,3.1");
}

#[test]
fn infill_jerk_applies_to_bridge_roles() {
    let bridge_output = bridge_jerk_output(
        PrintPathRole::Bridge,
        1,
        json!({
            "default_jerk": 8,
            "initial_layer_jerk": 0,
            "infill_jerk": 3
        }),
    );
    let internal_bridge_output = bridge_jerk_output(
        PrintPathRole::InternalBridge,
        1,
        json!({
            "default_jerk": 8,
            "initial_layer_jerk": 0,
            "infill_jerk": 4
        }),
    );

    assert_jerk_before_move(&bridge_output, "M205 X3 Y3", ";MOVE:print:bridge:1,0");
    assert_jerk_before_move(
        &internal_bridge_output,
        "M205 X4 Y4",
        ";MOVE:print:internal_bridge:1,0",
    );
}

#[tokio::test]
async fn travel_jerk_zero_does_not_fallback_to_default() {
    let output = slice_jerk_output(json!({
        "default_jerk": 8,
        "travel_jerk": 0
    }))
    .await;

    assert_no_jerk_before_move(&output, ";MOVE:travel:external_perimeter:-0.5,0");
    assert_jerk_before_move(&output, "M205 X9 Y9", ";MOVE:print:skirt:2.5,-2.5");
}

#[tokio::test]
async fn jerk_commands_are_suppressed_when_unchanged() {
    let output = slice_jerk_output(json!({
        "default_jerk": 8,
        "initial_layer_jerk": 8,
        "outer_wall_jerk": 8,
        "inner_wall_jerk": 8,
        "infill_jerk": 8,
        "travel_jerk": 8,
        "initial_layer_travel_jerk": "100%"
    }))
    .await;

    assert_eq!(jerk_lines(&output), vec!["M205 X8 Y8"]);
    assert_eq!(movement_command_count(&output), 27);
}

#[tokio::test]
async fn invalid_jerk_values_are_rejected() {
    for (key, value) in [
        ("default_jerk", json!(-1)),
        ("outer_wall_jerk", json!("bad")),
        ("inner_wall_jerk", json!(true)),
        ("infill_jerk", json!(-0.1)),
        ("initial_layer_jerk", json!({})),
        ("travel_jerk", json!("inf")),
        ("initial_layer_travel_jerk", json!("bad%")),
        ("initial_layer_travel_jerk", json!(-1)),
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
async fn gcode_comments_apply_to_jerk_commands() {
    let output = slice_jerk_output(json!({
        "gcode_comments": true,
        "default_jerk": 8,
        "initial_layer_jerk": 8,
        "travel_jerk": 8,
        "initial_layer_travel_jerk": "100%"
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "M205 X8 Y8 ; adjust jerk")
    );
}

#[tokio::test]
async fn default_junction_deviation_emits_marlin_firmware_m205_j_before_first_toolpath_move() {
    let output = slice_jerk_output(json!({
        "gcode_flavor": "marlin2",
        "emit_machine_limits_to_gcode": false,
        "default_junction_deviation": 0.025,
        "machine_max_junction_deviation": 0.1,
        "z_hop": 0
    }))
    .await;

    assert_eq!(
        first_non_comment_after_marker(&output, ";MOVE:travel:external_perimeter:-0.5,0"),
        Some("G1 X-0.5 Y0 F7200")
    );
    assert!(line_before(&output, "M205 J0.025", "G1 X-0.5 Y0 F7200"));
    assert_eq!(
        output.lines().filter(|line| *line == "M205 J0.025").count(),
        1
    );
}

#[tokio::test]
async fn default_junction_deviation_clamps_to_machine_maximum_in_gcode() {
    let output = slice_jerk_output(json!({
        "gcode_flavor": "marlin2",
        "emit_machine_limits_to_gcode": false,
        "default_junction_deviation": 0.25,
        "machine_max_junction_deviation": 0.1
    }))
    .await;

    assert!(output.lines().any(|line| line == "M205 J0.100"));
    assert!(!output.lines().any(|line| line == "M205 J0.250"));
}

#[tokio::test]
async fn default_junction_deviation_is_marlin_firmware_only() {
    for flavor in ["marlin", "klipper", "reprapfirmware", "repetier"] {
        let output = slice_jerk_output(json!({
            "gcode_flavor": flavor,
            "emit_machine_limits_to_gcode": false,
            "default_junction_deviation": 0.025,
            "machine_max_junction_deviation": 0.1
        }))
        .await;

        assert!(
            !output.lines().any(|line| line.starts_with("M205 J")),
            "{flavor}"
        );
    }
}

#[tokio::test]
async fn default_junction_deviation_comment_follows_gcode_comments() {
    let output = slice_jerk_output(json!({
        "gcode_flavor": "marlin2",
        "emit_machine_limits_to_gcode": false,
        "default_junction_deviation": 0.025,
        "machine_max_junction_deviation": 0.1,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "M205 J0.025 ; Junction Deviation")
    );
}

async fn slice_jerk_output(extra: serde_json::Value) -> String {
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

fn rectangle_jerk_output(extra: serde_json::Value) -> String {
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

fn bridge_jerk_output(role: PrintPathRole, layer_id: usize, extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "line_width": 0.4,
            "filament_diameter": [2.0],
            "bridge_speed": 20,
            "internal_bridge_speed": 20,
            "bridge_flow": 1.0,
            "internal_bridge_flow": 1.0
        }),
        extra,
    );
    let pipeline = crate::pipeline::test_support::single_path_pipeline(&options, role, layer_id);
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

fn jerk_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M205 X") && !line.contains(" Z"))
        .collect()
}

fn movement_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}

fn assert_jerk_before_move(output: &str, jerk: &str, marker: &str) {
    assert_eq!(first_non_comment_after_marker(output, marker), Some(jerk));
}

fn assert_no_jerk_before_move(output: &str, marker: &str) {
    assert!(
        !first_non_comment_after_marker(output, marker)
            .is_some_and(|line| line.starts_with("M205 "))
    );
}

fn line_before(output: &str, needle: &str, marker: &str) -> bool {
    let lines = output.lines().collect::<Vec<_>>();
    let needle_index = lines.iter().position(|line| *line == needle).unwrap();
    let marker_index = lines.iter().position(|line| *line == marker).unwrap();
    needle_index < marker_index
}

fn first_non_comment_after_marker<'a>(output: &'a str, marker: &str) -> Option<&'a str> {
    let lines = output.lines().collect::<Vec<_>>();
    let marker_index = lines.iter().position(|line| *line == marker).unwrap();
    lines[marker_index + 1..]
        .iter()
        .copied()
        .find(|line| !line.starts_with(';') && !line.starts_with("M204 "))
}
