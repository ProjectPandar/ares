#![cfg(target_os = "linux")]

//! OrcaSlicer.cpp:1775-1778,3805 initializes the CLI grid from ordered bed points.
use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{
        OrcaRunner,
        stages::{self, FailureKind, Stage},
    },
};
use std::{fs, path::Path};

#[test]
fn circle_plate_origin_matches_native_empty_selected_plate() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("circle-plate-origin-")
        .tempdir()
        .unwrap()
        .keep();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/validation");
    let input = crate::cli_retained_input::second_plate(&artifacts, &fixtures, "131.9");
    let native_dir = artifacts.join("native");
    fs::create_dir(&native_dir).unwrap();
    let failure = stages::run(
        runner.bin(),
        [
            "--slice".into(),
            "2".into(),
            "--outputdir".into(),
            native_dir.to_string_lossy().into_owned(),
            input.to_string_lossy().into_owned(),
        ],
        Stage::Slice,
        &artifacts,
    )
    .unwrap_err();
    assert_eq!(failure.kind, FailureKind::Process, "{failure}");
    let captured = failure.captured.unwrap();
    // The beta slices plate 2, writes its G-code and then rejects it from gcode_check_result:
    // CLI_GCODE_PATH_IN_UNPRINTABLE_AREA (-102, Utils.hpp:94; OrcaSlicer.cpp:7194-7204).
    assert_eq!(captured.code, Some(154));
    let native_json = fs::read(native_dir.join("result.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&native_json).unwrap();
    assert_eq!(parsed["return_code"], -102);
    assert_eq!(parsed["plate_index"], 2);
    assert!(native_dir.join("plate_2.gcode").exists());
    let output_path = artifacts.join("plate_2.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "--plate", "2", "-o"])
        .arg(&output_path)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("ares.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(
        output.status.code(),
        captured.code,
        "Native -102/154 unprintable-area rejection: {artifacts:?}"
    );
    assert_eq!(output.stdout, captured.stdout);
    assert_eq!(output.stderr, captured.stderr);
    let ares_json = fs::read(artifacts.join("result.json")).unwrap();
    let ares_parsed: serde_json::Value = serde_json::from_slice(&ares_json).unwrap();
    assert_eq!(
        ares_parsed["plate_index"], parsed["plate_index"],
        "Native -102 reports the actually selected plate: {artifacts:?}"
    );
    assert_eq!(
        result_comparison_bytes(&ares_json),
        result_comparison_bytes(&native_json)
    );
    assert!(output_path.exists());
}
