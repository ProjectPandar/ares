use std::fs;

use assert_cmd::Command;

pub(crate) fn slice_cube(options: &[u8]) -> String {
    let temp = tempfile::tempdir().unwrap();
    let options_path = temp.path().join("options.json");
    let input = temp.path().join("input.stl");
    let output = temp.path().join("output.gcode");
    fs::write(&options_path, options).unwrap();
    fs::write(
        &input,
        include_bytes!("../../../../tests/parity/cube10.stl"),
    )
    .unwrap();
    Command::cargo_bin("ares")
        .unwrap()
        .args(["slice", "--options"])
        .arg(options_path)
        .arg("-o")
        .arg(&output)
        .arg(input)
        .assert()
        .success();
    fs::read_to_string(output).unwrap()
}

pub(crate) fn deposits(line: &str) -> bool {
    line.starts_with("G1 X") && line.contains(" Y") && line.contains(" E")
}
