use super::*;

#[tokio::test]
async fn default_slice_reports_no_brim_artifacts() {
    let output = slice(square_pyramid_ascii_stl(), SliceOptions::default())
        .await
        .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_brim_path_count = 0")
    );
    assert!(!output.lines().any(|line| line.starts_with(";BRIM:")));
}

#[tokio::test]
async fn slice_emits_brim_artifacts_and_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_width": 1.2,
        "brim_object_gap": 0.2,
        "brim_type": "outer_only",
        "outer_wall_speed": 60,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_brim_path_count = 3")
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; brim_count = 3")
            .count(),
        1
    );
    assert!(output.lines().any(|line| line.starts_with(";BRIM:")));
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";PRINT_PATH:brim:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";MOVE:print:brim:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";EXTRUSION:print:brim:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";SPEED:print:brim:"))
    );
    assert!(output.lines().any(|line| line.contains(" F3600")));
    assert!(standalone_feedrate_command_count(&output) > 0);
}

fn standalone_feedrate_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with("G1 F"))
        .count()
}
