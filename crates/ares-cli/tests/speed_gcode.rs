use std::fs;

use assert_cmd::Command;

#[test]
fn slice_stl_writes_speed_feedrates() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(
        &options,
        br#"{"layer_height":0.2,"initial_layer_height":0.2,"seam_gap":0,"sparse_infill_density":50,"sparse_infill_line_width":0.25,"minimum_sparse_infill_area":0,"infill_direction":0,"is_infill_first":true,"infill_anchor_max":0,"filament_max_volumetric_speed":0.0,"slow_down_for_layer_cooling":false}"#,
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
    assert!(
        gcode
            .lines()
            .any(|line| line == "; total_speed_move_count = 27")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SPEED:travel:external_perimeter:-0.5,0:7200")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SPEED:print:external_perimeter:-0.5,0:1800")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SPEED:print:bottom_surface:-0.391421,0.108579:3600")
    );
    assert!(gcode.lines().any(|line| line == "G1 X-0.5 Y0 Z0.6 F7200"));
    assert!(gcode.lines().any(|line| line == "G1 X-0.5 Y0 E0.02393"));
    assert!(
        gcode
            .lines()
            .any(|line| line == "G1 X-0.391 Y0.109 E0.02394")
    );
    assert_eq!(path_following_command_count(&gcode), 27);
    assert!(standalone_feedrate_command_count(&gcode) > 0);
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

fn square_pyramid_ascii_stl() -> Vec<u8> {
    b"solid pyramid\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.4\nvertex 0 1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.4\nvertex 1 0 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.4\nvertex 0 -1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.4\nvertex -1 0 0.4\nendloop\nendfacet\nendsolid pyramid"
        .to_vec()
}
