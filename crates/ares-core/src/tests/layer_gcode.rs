use super::*;

#[tokio::test]
async fn slice_accepts_percent_sparse_infill_width_with_extrusion_output() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 0,
        "sparse_infill_line_width": "120%"
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; total_extrusion_move_count = "));
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("G1 X") && line.contains(" E"))
    );
}

#[tokio::test]
async fn slice_emits_layer_aware_gcode() {
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
        "top_shell_layers": 0
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; layer_height = 0.2"));
    assert!(output.contains("; initial_layer_height = 0.2"));
    assert!(output.contains("; layer_count = 2"));
    assert!(output.lines().any(|line| line
        == "; pipeline_stages = model,layers,segments,contours,perimeters,infills,skirts,brims,print_paths,moves,extrusions,speeds"));
    assert!(
        output
            .lines()
            .any(|line| line == "; total_segment_count = 8")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_contour_count = 2")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_perimeter_count = 2")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_infill_count = 6")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_print_path_count = 9")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_toolpath_move_count = 27")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_extrusion_move_count = 27")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; total_extrusion_mm = 1.050284")
    );
    assert!(output.lines().any(|line| line == "; empty_layer_count = 0"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.2\nG1 Z0.2"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:1\n;Z:0.4\nG1 Z0.4"));
    assert!(output.contains("; segment_count = 4"));
    assert!(output.contains("; contour_count = 1"));
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; perimeter_count = 1")
            .count(),
        2
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; infill_count = 2")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; infill_count = 4")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; print_path_count = 4")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; print_path_count = 5")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; toolpath_move_count = 14")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; toolpath_move_count = 13")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; extrusion_move_count = 14")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; extrusion_move_count = 13")
            .count(),
        1
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; extrusion_mm = 0.789934")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "; extrusion_mm = 0.26035")
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";INFILL:sparse:"))
            .count(),
        6
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";PRINT_PATH:sparse_infill:"))
            .count(),
        6
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";PRINT_PATH:external_perimeter:"))
            .count(),
        2
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";MOVE:travel:"))
            .count(),
        9
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";MOVE:print:"))
            .count(),
        18
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";EXTRUSION:travel:"))
            .count(),
        9
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with(";EXTRUSION:print:"))
            .count(),
        18
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";INFILL:sparse:-0.25,-0.25 -> -0.25,0.25")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";INFILL:sparse:0.25,-0.25 -> 0.25,0.25")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";INFILL:sparse:0.25,-0.75 -> -0.25,-0.75")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:sparse_infill:-0.25,-0.25 -> -0.25,0.25")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:travel:external_perimeter:-0.5,0")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:print:external_perimeter:-0.5,0")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:travel:sparse_infill:-0.25,-0.25")
    );
    assert_move_command_pair(
        &output,
        ";MOVE:travel:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 Z0.6 F7200",
    );
    assert_move_command_pair(
        &output,
        ";MOVE:print:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 E0.02393",
    );
    assert_extrusion_move_command_block(
        &output,
        ";EXTRUSION:travel:external_perimeter:-0.5,0:",
        "G1 X-0.5 Y0 Z0.6 F7200",
    );
    assert_extrusion_move_command_block(
        &output,
        ";EXTRUSION:print:external_perimeter:-0.5,0:0.772716",
        "G1 X-0.5 Y0 E0.02393",
    );
    assert_extrusion_move_command_block(
        &output,
        ";EXTRUSION:print:sparse_infill:0.25,0.25:0.789934",
        "G1 X0.25 Y0.25 E0.00861",
    );
    assert_extrusion_move_command_block(
        &output,
        ";EXTRUSION:print:external_perimeter:-1,0:1.050284",
        "G1 X-1 Y0 E0.04787",
    );
    assert_move_command_pair(
        &output,
        ";MOVE:travel:sparse_infill:-0.25,-0.25",
        "G1 X-0.25 Y-0.25 F7200",
    );
    assert_eq!(move_command_count(&output, "G0"), 0);
    assert_eq!(move_command_count(&output, "G1"), 27);
    assert_eq!(path_following_command_count(&output), 27);
    assert_eq!(print_move_commands_without_e(&output), 0);
    assert_move_commands_have_extrusion_contract(&output);
    assert!(output.lines().any(
        |line| line == ";PRINT_PATH:external_perimeter:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"
    ));
    let layer0_external = output
        .lines()
        .position(|line| {
            line == ";PRINT_PATH:external_perimeter:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"
        })
        .unwrap();
    let layer0_sparse = output
        .lines()
        .position(|line| line == ";PRINT_PATH:sparse_infill:-0.25,-0.25 -> -0.25,0.25")
        .unwrap();
    assert!(layer0_external < layer0_sparse);
    let layer1_sparse = output
        .lines()
        .position(|line| line == ";PRINT_PATH:sparse_infill:0.25,-0.75 -> -0.25,-0.75")
        .unwrap();
    let layer1_external = output
        .lines()
        .position(|line| line == ";PRINT_PATH:external_perimeter:-1,0 -> 0,-1 -> 1,0 -> 0,1")
        .unwrap();
    assert!(layer1_sparse < layer1_external);
    assert!(output.contains(";CONTOUR:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"));
    assert!(output.contains(";PERIMETER:external:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"));
    assert!(output.contains(";SEGMENT:-0.5,0 -> 0,0.5"));
    assert!(output.contains("G1 X-0.5 Y0 Z0.6 F7200"));
    assert!(!output.contains("G0 X0 Y0.5"));
}

fn assert_extrusion_move_command_block(output: &str, marker: &str, command: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let index = lines.iter().position(|line| *line == marker).unwrap();
    let move_marker = matching_move_marker(marker);
    assert_eq!(lines.get(index + 1), Some(&move_marker.as_str()));
    assert_eq!(movement_command_after(&lines, index + 1), command);
}

fn matching_move_marker(extrusion_marker: &str) -> String {
    extrusion_marker
        .replacen(";EXTRUSION:", ";MOVE:", 1)
        .rsplit_once(':')
        .unwrap()
        .0
        .to_owned()
}

fn assert_move_commands_have_extrusion_contract(output: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    for (index, line) in lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:"))
    {
        let command = movement_command_after(&lines, index);
        if line.starts_with(";MOVE:print:") {
            assert!(command.starts_with("G1"));
            assert!(command.contains(" E"));
        } else if line.starts_with(";MOVE:travel:") {
            assert!(command.starts_with("G1"));
            assert!(!command.contains(" E"));
        }
    }
}

fn path_following_command_count(output: &str) -> usize {
    move_command_count(output, "G0") + move_command_count(output, "G1")
}

fn print_move_commands_without_e(output: &str) -> usize {
    let lines = output.lines().collect::<Vec<_>>();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:print:"))
        .filter(|(index, _)| !movement_command_after(&lines, *index).contains(" E"))
        .count()
}

fn assert_move_command_pair(output: &str, marker: &str, command: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let index = lines.iter().position(|line| *line == marker).unwrap();
    assert_eq!(movement_command_after(&lines, index), command);
}

fn move_command_count(output: &str, prefix: &str) -> usize {
    let lines = output.lines().collect::<Vec<_>>();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:"))
        .filter(|(index, _)| movement_command_after(&lines, *index).starts_with(prefix))
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
