use super::*;

#[tokio::test]
async fn slice_emits_speed_feedrates_on_path_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "seam_gap": 0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_speed_move_count = 27")
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; speed_move_count = 14")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; speed_move_count = 13")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";SPEED:travel:"))
            .count(),
        9
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";SPEED:print:"))
            .count(),
        18
    );
    assert_speed_move_command_block(
        &output,
        ";SPEED:travel:external_perimeter:-0.5,0:7200",
        ";EXTRUSION:travel:external_perimeter:-0.5,0:",
        ";MOVE:travel:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 Z0.6 F7200",
    );
    assert_speed_move_command_block(
        &output,
        ";SPEED:print:external_perimeter:-0.5,0:1800",
        ";EXTRUSION:print:external_perimeter:-0.5,0:0.772716",
        ";MOVE:print:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 E0.02393",
    );
    assert_speed_move_command_block(
        &output,
        ";SPEED:print:sparse_infill:0.25,0.25:3600",
        ";EXTRUSION:print:sparse_infill:0.25,0.25:0.789934",
        ";MOVE:print:sparse_infill:0.25,0.25",
        "G1 X0.25 Y0.25 E0.00861",
    );
    assert_eq!(path_following_command_count(&output), 27);
    assert!(standalone_feedrate_command_count(&output) > 0);
}

fn assert_speed_move_command_block(
    output: &str,
    speed: &str,
    extrusion: &str,
    marker: &str,
    command: &str,
) {
    let lines = output.lines().collect::<Vec<_>>();
    let index = lines.iter().position(|line| *line == speed).unwrap();
    assert_eq!(lines.get(index + 1), Some(&extrusion));
    assert_eq!(lines.get(index + 2), Some(&marker));
    assert_eq!(movement_command_after(&lines, index + 2), command);
}

fn path_following_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}

fn standalone_feedrate_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with("G1 F"))
        .count()
}

fn movement_command_after<'a>(lines: &'a [&str], marker_index: usize) -> &'a str {
    lines[marker_index + 1..]
        .iter()
        .copied()
        .take_while(|line| !line.starts_with(';'))
        .find(|line| line.starts_with("G1 X") || line.starts_with("G1 Y"))
        .unwrap()
}
