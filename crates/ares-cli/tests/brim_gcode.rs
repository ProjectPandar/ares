use std::fs;

use assert_cmd::Command;

#[test]
fn brim_gcode_slice_stl_writes_artifacts_and_commands() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(
        &options,
        br#"{"layer_height":0.2,"initial_layer_height":0.2,"brim_width":1.2,"brim_object_gap":0.2,"brim_type":"outer_only","outer_wall_speed":60,"filament_max_volumetric_speed":0.0,"slow_down_for_layer_cooling":false}"#,
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
            .any(|line| line == "; total_brim_path_count = 3")
    );
    assert!(gcode.lines().any(|line| line == "; brim_count = 3"));
    assert!(gcode.lines().any(|line| line.starts_with(";BRIM:")));
    assert!(
        gcode
            .lines()
            .any(|line| line.starts_with(";PRINT_PATH:brim:"))
    );
    assert!(
        gcode
            .lines()
            .any(|line| line.starts_with(";MOVE:print:brim:"))
    );
    assert!(
        gcode
            .lines()
            .any(|line| line.starts_with(";EXTRUSION:print:brim:"))
    );
    assert!(
        gcode
            .lines()
            .any(|line| line.starts_with(";SPEED:print:brim:"))
    );
    assert!(gcode.lines().any(|line| line.contains(" F3600")));
    assert!(standalone_feedrate_command_count(&gcode) > 0);
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
