use super::*;

#[tokio::test]
async fn gcode_comments_default_does_not_add_inline_command_comments() {
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
    let omitted = String::from_utf8(omitted).unwrap();

    assert!(omitted.lines().any(|line| line.starts_with("G1 ")));
    assert!(!omitted.contains(" ; move to layer Z"));
    assert!(!omitted.contains(" ; perimeter"));
}

#[tokio::test]
async fn gcode_comments_false_preserves_command_lines() {
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
    let disabled = String::from_utf8(
        slice(
            square_pyramid_ascii_stl(),
            serde_json::from_value(json!({
                "sparse_infill_density": 0,
                "gcode_comments": false
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    )
    .unwrap();

    assert_eq!(command_lines(&omitted), command_lines(&disabled));
    assert!(omitted.lines().any(|line| line == "; option_count = 1"));
    assert!(disabled.lines().any(|line| line == "; option_count = 2"));
}

#[tokio::test]
async fn gcode_comments_true_adds_inline_command_comments_only() {
    let commented = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.25,
            "minimum_sparse_infill_area": 0,
            "infill_direction": 0,
            "is_infill_first": true,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false,
            "z_hop": 0,
            "seam_gap": 0,
            "gcode_comments": true
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let commented = String::from_utf8(commented).unwrap();

    assert!(
        commented
            .lines()
            .any(|line| line == "G1 Z0.2 F7200 ; move to layer Z")
    );
    assert!(
        commented
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(
        commented
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        commented
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
    assert!(
        commented
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 E0.02393 ; extrude")
    );
    assert_eq!(
        commented
            .lines()
            .filter(|line| line.starts_with(";SPEED:travel:"))
            .count(),
        9
    );
    assert_eq!(
        commented
            .lines()
            .filter(|line| line.starts_with(";SPEED:print:"))
            .count(),
        18
    );
    assert_eq!(path_following_command_count(&commented), 27);
}

#[tokio::test]
async fn gcode_comments_rejects_non_boolean_values() {
    let err = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(json!({
            "gcode_comments": "true"
        }))
        .unwrap(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("gcode_comments must be a boolean"));
}

fn path_following_command_count(output: &str) -> usize {
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
