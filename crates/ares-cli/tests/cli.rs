use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn slice_accepts_percent_sparse_infill_width() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(
        &options,
        br#"{"sparse_infill_density":0,"sparse_infill_line_width":"120%"}"#,
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
            .any(|line| line.starts_with("G1 X") && line.contains(" E"))
    );
}

#[test]
fn slice_3mf_fails_until_geometry_extraction_exists() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.3mf");
    let output = temp.path().join("output.gcode");
    fs::write(&options, b"{}").unwrap();
    fs::write(&input, b"PK\x03\x04fake-3mf").unwrap();

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
        .failure()
        .stderr(predicate::str::contains("model has no triangles"));
}

#[test]
fn slice_rejects_obj_input() {
    let temp = tempfile::tempdir().unwrap();
    let options = temp.path().join("option.json");
    let input = temp.path().join("input.obj");
    let output = temp.path().join("output.gcode");
    fs::write(&options, b"{}").unwrap();
    fs::write(&input, b"o cube\n").unwrap();

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
        .failure()
        .stderr(predicate::str::contains("unsupported input extension"));
}

fn square_pyramid_ascii_stl() -> Vec<u8> {
    b"solid pyramid\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.4\nvertex 0 1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.4\nvertex 1 0 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.4\nvertex 0 -1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.4\nvertex -1 0 0.4\nendloop\nendfacet\nendsolid pyramid"
        .to_vec()
}
