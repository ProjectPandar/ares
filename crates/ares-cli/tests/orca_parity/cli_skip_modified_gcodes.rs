use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
    process::Command,
};

use crate::{
    load_settings_cli::{banner_normalized, stages_run},
    runner::{OrcaRunner, repo_root},
};

const CUSTOM: &str = "M117 Saved custom event";
const PLATE_EVENTS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<custom_gcodes_per_layer>
  <plate>
    <plate_info id="1"/>
    <layer top_z="0.6" type="1" extruder="1" color="" extra="Pause"/>
    <layer top_z="1" type="4" extruder="1" color="" extra="M117 Saved custom event"/>
  </plate>
</custom_gcodes_per_layer>"#;

#[test]
fn cli_skip_modified_gcodes_keeps_pause_and_removes_only_custom_events() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let temp = tempfile::Builder::new()
        .prefix("skip-modified-gcodes-")
        .clone();
    let directory = match std::env::var_os("ARES_PARITY_ARTIFACT_ROOT") {
        Some(root) => temp.tempdir_in(root),
        None => temp.tempdir(),
    }
    .unwrap()
    .keep();
    let control = fs::read(repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let edited = with_plate_events(&control);
    let off = native(&runner, &directory.join("omitted"), &edited, &[]);
    let explicit_off = native(
        &runner,
        &directory.join("false"),
        &edited,
        &["--skip-modified-gcodes=false"],
    );
    let on = native(
        &runner,
        &directory.join("enabled"),
        &edited,
        &["--skip-modified-gcodes"],
    );
    for (gcode, keeps_custom) in [(&off, true), (&on, false)] {
        let text = std::str::from_utf8(gcode).unwrap();
        assert!(text.lines().any(|line| line == "M25"));
        assert_eq!(text.lines().any(|line| line == CUSTOM), keeps_custom);
    }
    assert!(banner_normalized(&off) == banner_normalized(&explicit_off));
    let untouched = native(&runner, &directory.join("control"), &control, &[]);
    let untouched_on = native(
        &runner,
        &directory.join("control-enabled"),
        &control,
        &["--skip-modified-gcodes"],
    );
    assert!(banner_normalized(&untouched) == banner_normalized(&untouched_on));

    for (name, flags, expected) in [
        ("omitted", [].as_slice(), off),
        ("enabled", ["--skip-modified-gcodes"].as_slice(), on),
        (
            "false",
            ["--skip-modified-gcodes=false"].as_slice(),
            explicit_off,
        ),
        ("control", [].as_slice(), untouched),
        (
            "control-enabled",
            ["--skip-modified-gcodes"].as_slice(),
            untouched_on,
        ),
    ] {
        ares(&directory.join(name), flags, &expected);
    }
}

fn native(runner: &OrcaRunner, directory: &Path, project: &[u8], flags: &[&str]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
    fs::write(directory.join("project.3mf"), project).unwrap();
    let args = [
        ["--slice", "0", "--arrange", "0", "--outputdir", "native"].as_slice(),
        flags,
        ["project.3mf"].as_slice(),
    ]
    .concat();
    fs::write(
        directory.join("native-argv.json"),
        serde_json::to_vec(&args).unwrap(),
    )
    .unwrap();
    stages_run(runner, directory, &args);
    fs::read(directory.join("native/plate_1.gcode")).unwrap()
}

fn ares(directory: &Path, flags: &[&str], native: &[u8]) {
    fs::create_dir(directory.join("ares")).unwrap();
    let args = [
        ["slice", "-o", "ares/plate_1.gcode"].as_slice(),
        flags,
        ["project.3mf"].as_slice(),
    ]
    .concat();
    fs::write(
        directory.join("ares-argv.json"),
        serde_json::to_vec(&args).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(&args)
        .current_dir(directory)
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "Ares {}: {:?}",
        output.status,
        directory
    );
    let actual = fs::read(directory.join("ares/plate_1.gcode")).unwrap();
    assert!(
        banner_normalized(&actual) == banner_normalized(native),
        "complete G-code differs: {directory:?}"
    );
}

fn with_plate_events(project: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        output
            .raw_copy_file(input.by_index(index).unwrap())
            .unwrap();
    }
    output
        .start_file(
            "Metadata/custom_gcode_per_layer.xml",
            zip::write::SimpleFileOptions::DEFAULT,
        )
        .unwrap();
    output.write_all(PLATE_EVENTS.as_bytes()).unwrap();
    output.finish().unwrap().into_inner()
}
