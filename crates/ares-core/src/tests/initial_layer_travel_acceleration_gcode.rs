use super::*;

#[tokio::test]
async fn first_layer_travel_acceleration_uses_absolute_value() {
    let output = slice_initial_layer_travel_acceleration_output(json!({
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "initial_layer_travel_acceleration": 420
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "M204 S420",
        ";MOVE:travel:external_perimeter:-0.5,0",
    );
    assert_acceleration_before_move(
        &output,
        "M204 S900",
        ";MOVE:travel:sparse_infill:-0.75,-0.25",
    );
}

#[tokio::test]
async fn first_layer_travel_acceleration_percent_uses_travel_base() {
    let output = slice_initial_layer_travel_acceleration_output(json!({
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "initial_layer_travel_acceleration": "50%"
    }))
    .await;

    assert_acceleration_before_move(
        &output,
        "M204 S450",
        ";MOVE:travel:external_perimeter:-0.5,0",
    );
}

#[tokio::test]
async fn first_layer_travel_acceleration_zero_suppresses_first_layer_only() {
    let output = slice_initial_layer_travel_acceleration_output(json!({
        "default_acceleration": 700,
        "travel_acceleration": 900,
        "initial_layer_travel_acceleration": 0
    }))
    .await;

    assert_no_acceleration_before_move(&output, ";MOVE:travel:external_perimeter:-0.5,0");
    assert_acceleration_before_move(&output, "M204 S300", ";MOVE:print:skirt:2.5,-2.5");
    assert_acceleration_before_move(
        &output,
        "M204 S900",
        ";MOVE:travel:sparse_infill:-0.75,-0.25",
    );
}

#[tokio::test]
async fn invalid_initial_layer_travel_acceleration_values_are_rejected() {
    for value in [json!("bad%"), json!(-1), json!(false)] {
        let err = slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({
                "initial_layer_travel_acceleration": value
            }))
            .unwrap(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

async fn slice_initial_layer_travel_acceleration_output(extra: serde_json::Value) -> String {
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

fn assert_no_acceleration_before_move(output: &str, marker: &str) {
    assert!(
        !first_non_comment_after_marker(output, marker)
            .is_some_and(|line| line.starts_with("M204 "))
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
