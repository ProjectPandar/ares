use {assert_cmd::Command, predicates::prelude::*, std::fs};

mod slice_gcode_support;

use slice_gcode_support::*;
#[test]
fn slice_stl_writes_output_with_stl_format() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(
        &options,
        br#"{"layer_height":0.2,"initial_layer_height":0.2,"seam_gap":0,"nozzle_diameter":["0.4"],"filament_diameter":["1.75"],"min_layer_height":["0.07"],"max_layer_height":["0.28"],"sparse_infill_density":50,"sparse_infill_line_width":0.25,"minimum_sparse_infill_area":0,"infill_direction":0,"is_infill_first":true,"infill_anchor_max":0}"#,
    )
    .unwrap();
    fs::write(&input, square_pyramid_ascii_stl()).unwrap();

    Command::cargo_bin("ares")
        .unwrap()
        .args([
            "slice",
            "--options",
            options.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .assert()
        .success();

    let gcode = fs::read_to_string(output).unwrap();
    assert!(predicate::str::contains("input_format = stl").eval(&gcode));
    assert!(predicate::str::contains("triangle_count = 4").eval(&gcode));
    assert!(predicate::str::contains("layer_count = 2").eval(&gcode));
    assert!(gcode.lines().any(|line| line
        == "; pipeline_stages = model,layers,segments,contours,perimeters,infills,skirts,brims,print_paths,moves,extrusions,speeds"));
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_segment_count = 8")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_contour_count = 2")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_perimeter_count = 2")
    );
    assert!(gcode.lines().any(|line| line == "; total_infill_count = 6"));
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_print_path_count = 9")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_toolpath_move_count = 27")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_extrusion_move_count = 27")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_extrusion_mm = 1.203538")
    );
    assert!(gcode.lines().any(|line| line == "; empty_layer_count = 0"));
    assert!(predicate::str::contains("nozzle_diameter = 0.4").eval(&gcode));
    assert!(predicate::str::contains("filament_diameter = 1.75").eval(&gcode));
    assert!(predicate::str::contains("min_layer_height = 0.07").eval(&gcode));
    assert!(predicate::str::contains("max_layer_height = 0.28").eval(&gcode));
    assert!(predicate::str::contains(";LAYER:0").eval(&gcode));
    assert!(predicate::str::contains("G1 Z0.4").eval(&gcode));
    assert!(gcode.lines().any(|line| line == "; segment_count = 4"));
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; contour_count = 1")
            .count(),
        2
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; perimeter_count = 1")
            .count(),
        2
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; infill_count = 2")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; infill_count = 4")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; print_path_count = 4")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; print_path_count = 5")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; toolpath_move_count = 14")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; toolpath_move_count = 13")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; extrusion_move_count = 14")
            .count(),
        1
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| *line == "; extrusion_move_count = 13")
            .count(),
        1
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; extrusion_mm = 0.820586")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == "; extrusion_mm = 0.382952")
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";INFILL:solid:"))
            .count(),
        6
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";PRINT_PATH:bottom_surface:"))
            .count(),
        6
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";PRINT_PATH:external_perimeter:"))
            .count(),
        2
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";MOVE:travel:"))
            .count(),
        9
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";MOVE:print:"))
            .count(),
        18
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";EXTRUSION:travel:"))
            .count(),
        9
    );
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";EXTRUSION:print:"))
            .count(),
        18
    );
    assert!(predicate::str::contains(";SEGMENT:").eval(&gcode));
    assert!(predicate::str::contains(";CONTOUR:").eval(&gcode));
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";PERIMETER:external:"))
            .count(),
        2
    );
    assert!(
        gcode
            .lines()
            .any(|line| { line == ";PERIMETER:external:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5" })
    );
    assert!(
        gcode
            .lines()
            .any(|line| { line == ";PERIMETER:external:-1,0 -> 0,-1 -> 1,0 -> 0,1" })
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";INFILL:solid:0.108579,-0.391421 -> -0.391421,0.108579")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";INFILL:solid:0.391421,-0.108579 -> -0.108579,0.391421")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";INFILL:solid:0.924264,0.075736 -> -0.075736,-0.924264")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line
                == ";PRINT_PATH:bottom_surface:0.108579,-0.391421 -> -0.391421,0.108579")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";MOVE:travel:external_perimeter:-0.5,0")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";MOVE:print:external_perimeter:-0.5,0")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";MOVE:travel:bottom_surface:0.108579,-0.391421")
    );
    assert_move_command_pair(
        &gcode,
        ";MOVE:travel:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 Z0.6 F7200",
    );
    assert_move_command_pair(
        &gcode,
        ";MOVE:print:external_perimeter:-0.5,0",
        "G1 X-0.5 Y0 E0.02393",
    );
    assert_extrusion_move_command_block(
        &gcode,
        ";EXTRUSION:travel:external_perimeter:-0.5,0:",
        "G1 X-0.5 Y0 Z0.6 F7200",
    );
    assert_extrusion_move_command_block(
        &gcode,
        ";EXTRUSION:print:external_perimeter:-0.5,0:0.772716",
        "G1 X-0.5 Y0 E0.02393",
    );
    assert_extrusion_move_command_block(
        &gcode,
        ";EXTRUSION:print:bottom_surface:-0.391421,0.108579:0.796651",
        "G1 X-0.391 Y0.109 E0.02394",
    );
    assert_extrusion_move_command_block(
        &gcode,
        ";EXTRUSION:print:external_perimeter:-1,0:1.203538",
        "G1 X-1 Y0 E0.04787",
    );
    assert_move_command_pair(
        &gcode,
        ";MOVE:travel:bottom_surface:0.108579,-0.391421",
        "G1 X0.109 Y-0.391 F7200",
    );
    assert_eq!(move_command_count(&gcode, "G0"), 0);
    assert_eq!(move_command_count(&gcode, "G1"), 27);
    assert_eq!(path_following_command_count(&gcode), 27);
    assert_eq!(print_move_commands_without_e(&gcode), 0);
    assert_move_commands_have_extrusion_contract(&gcode);
    assert!(gcode.lines().any(
        |line| line == ";PRINT_PATH:external_perimeter:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"
    ));
    let layer0_external = gcode
        .lines()
        .position(|line| {
            line == ";PRINT_PATH:external_perimeter:-0.5,0 -> 0,-0.5 -> 0.5,0 -> 0,0.5"
        })
        .unwrap();
    let layer0_sparse = gcode
        .lines()
        .position(|line| {
            line == ";PRINT_PATH:bottom_surface:0.108579,-0.391421 -> -0.391421,0.108579"
        })
        .unwrap();
    assert!(layer0_external < layer0_sparse);
    let layer1_sparse = gcode
        .lines()
        .position(|line| {
            line == ";PRINT_PATH:bottom_surface:0.924264,0.075736 -> -0.075736,-0.924264"
        })
        .unwrap();
    let layer1_external = gcode
        .lines()
        .position(|line| line == ";PRINT_PATH:external_perimeter:-1,0 -> 0,-1 -> 1,0 -> 0,1")
        .unwrap();
    assert!(layer1_sparse < layer1_external);
}
