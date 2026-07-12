use super::*;

#[tokio::test]
async fn z_offset_default_preserves_existing_output_bytes() {
    let omitted = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(omitted.len(), 4753);
    assert_eq!(fnv1a64(&omitted), 0x8990a54281eb9dfd);
}

#[tokio::test]
async fn z_offset_zero_preserves_command_lines() {
    let omitted = String::from_utf8(
        slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({
                "sparse_infill_density": 0
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    let zero = String::from_utf8(
        slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({
                "sparse_infill_density": 0,
                "z_offset": 0
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    )
    .unwrap();

    assert_eq!(command_lines(&omitted), command_lines(&zero));
    assert!(omitted.lines().any(|line| line == "; option_count = 1"));
    assert!(zero.lines().any(|line| line == "; option_count = 2"));
}

#[tokio::test]
async fn z_offset_positive_offsets_only_layer_z_commands() {
    let output = slice_offset_output(json!(0.15), json!({ "z_hop": 0, "seam_gap": 0 })).await;

    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.2\nG1 Z0.35 F7200"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:1\n;Z:0.4\nG1 Z0.55 F7200"));
    assert_eq!(
        layer_z_command_lines(&output),
        vec!["G1 Z0.35 F7200", "G1 Z0.55 F7200"]
    );
    assert_eq!(path_following_command_count(&output), 27);
    assert_eq!(move_diagnostic_count(&output), 27);
    assert!(output.lines().any(|line| line == "G1 X-0.5 Y0 F7200"));
    assert!(output.lines().any(|line| line == "G1 X-0.5 Y0 E0.02393"));
}

#[tokio::test]
async fn z_offset_negative_offsets_only_layer_z_commands() {
    let output = slice_offset_output(json!(-0.05), json!({})).await;

    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.2\nG1 Z0.15 F7200"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:1\n;Z:0.4\nG1 Z0.35 F7200"));
    assert_eq!(
        layer_z_command_lines(&output),
        vec!["G1 Z0.15 F7200", "G1 Z0.35 F7200"]
    );
    assert_eq!(path_following_command_count(&output), 27);
    assert_eq!(move_diagnostic_count(&output), 27);
}

#[tokio::test]
async fn z_offset_accepts_numeric_string_values() {
    let output = slice_offset_output(json!("0.15"), json!({})).await;

    assert_eq!(
        layer_z_command_lines(&output),
        vec!["G1 Z0.35 F7200", "G1 Z0.55 F7200"]
    );
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.2\nG1 Z0.35 F7200"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:1\n;Z:0.4\nG1 Z0.55 F7200"));
}

#[tokio::test]
async fn z_offset_rejects_non_numeric_values() {
    let err = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "z_offset": true
        }))
        .unwrap(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("z_offset must be a number"));
}

#[tokio::test]
async fn z_offset_rejects_non_finite_values() {
    let err = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "z_offset": "inf"
        }))
        .unwrap(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("z_offset is out of range"));
}

#[tokio::test]
async fn z_offset_composes_with_gcode_comments() {
    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.25,
            "minimum_sparse_infill_area": 0,
            "infill_direction": 0,
            "is_infill_first": true,
            "z_offset": 0.15,
            "z_hop": 0,
            "seam_gap": 0,
            "gcode_comments": true
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "G1 Z0.35 F7200 ; move to layer Z")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "G1 Z0.55 F7200 ; move to layer Z")
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

async fn slice_offset_output(offset: serde_json::Value, extra: serde_json::Value) -> String {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "z_offset": offset
    });
    options
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(options).unwrap(),
    )
    .await
    .unwrap();

    String::from_utf8(output).unwrap()
}

fn layer_z_command_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .collect::<Vec<_>>()
        .windows(4)
        .filter_map(|window| {
            (window[0] == ";LAYER_CHANGE"
                && window[1].starts_with(";LAYER:")
                && window[2].starts_with(";Z:"))
            .then_some(window[3])
        })
        .collect()
}

fn path_following_command_count(output: &str) -> usize {
    let mut pending_move = false;
    let mut count = 0;
    for line in output.lines() {
        if line.starts_with(";MOVE:") {
            pending_move = true;
        } else if pending_move && line.starts_with("G1 X") {
            count += 1;
            pending_move = false;
        } else if pending_move && line.starts_with(';') {
            pending_move = false;
        }
    }
    count
}

fn move_diagnostic_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}

fn command_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect()
}
