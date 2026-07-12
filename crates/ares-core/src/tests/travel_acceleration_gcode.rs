use super::*;

#[tokio::test]
async fn marlin2_travel_acceleration_uses_separate_travel_command() {
    let output = slice_travel_acceleration_output(json!({
        "gcode_flavor": "marlin2",
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }))
    .await;

    assert_acceleration_before_move(&output, "M204 T900", ";MOVE:travel:skirt:-2.5,-2.5");
    assert_acceleration_before_move(&output, "M204 P350", ";MOVE:print:sparse_infill:-0.75,0.25");
}

#[tokio::test]
async fn repetier_travel_acceleration_uses_m202_and_print_uses_m201() {
    let output = slice_travel_acceleration_output(json!({
        "gcode_flavor": "repetier",
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }))
    .await;

    assert_acceleration_before_move(&output, "M202 X900 Y900", ";MOVE:travel:skirt:-2.5,-2.5");
    assert_acceleration_before_move(
        &output,
        "M201 X350 Y350",
        ";MOVE:print:sparse_infill:-0.75,0.25",
    );
}

#[tokio::test]
async fn reprap_firmware_travel_acceleration_uses_separate_travel_command() {
    let output = slice_travel_acceleration_output(json!({
        "gcode_flavor": "reprapfirmware",
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "sparse_infill_acceleration": "50%"
    }))
    .await;

    assert_acceleration_before_move(&output, "M204 T900", ";MOVE:travel:skirt:-2.5,-2.5");
    assert_acceleration_before_move(&output, "M204 P350", ";MOVE:print:sparse_infill:-0.75,0.25");
}

#[tokio::test]
async fn marlin2_first_layer_travel_acceleration_uses_separate_travel_command() {
    let output = slice_travel_acceleration_output(json!({
        "gcode_flavor": "marlin2",
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "initial_layer_travel_acceleration": 420
    }))
    .await;

    assert_acceleration_before_move(&output, "M204 T420", ";MOVE:travel:skirt:-2.5,-2.5");
    assert_acceleration_before_move(
        &output,
        "M204 T900",
        ";MOVE:travel:sparse_infill:-0.75,-0.25",
    );
}

async fn slice_travel_acceleration_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
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

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}

fn assert_acceleration_before_move(output: &str, acceleration: &str, marker: &str) {
    assert_eq!(
        first_non_comment_after_marker(output, marker),
        Some(acceleration)
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
