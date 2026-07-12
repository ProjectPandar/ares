use std::fs;

use assert_cmd::Command;

#[test]
fn slice_stl_writes_skirt_artifacts_and_commands() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(
        &options,
        br#"{"layer_height":0.2,"initial_layer_height":0.2,"sparse_infill_density":50,"sparse_infill_line_width":0.25,"minimum_sparse_infill_area":0,"infill_direction":0,"is_infill_first":true,"filament_max_volumetric_speed":0.0,"slow_down_for_layer_cooling":false}"#,
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
            .any(|line| line == "; total_skirt_path_count = 1")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SPEED:print:skirt:2.5,-2.5:3000")
    );
    assert!(
        gcode
            .lines()
            .any(|line| { line.starts_with("G1 X2.5 Y-2.5 E") })
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
