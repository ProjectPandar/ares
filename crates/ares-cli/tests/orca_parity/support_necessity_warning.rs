//! `PrintObject.cpp:966-986,5289-5303` and `OrcaSlicer.cpp:7148-7160` at beta345efec64.
//! A successful real slice reports the cantilever in result.json, not stderr.

use std::{fs, path::Path};

use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, stages},
};

#[test]
fn cli_success_reports_support_necessity_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("cli-support-necessity-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    eprintln!("support necessity: artifacts {artifacts:?}");
    let project = artifacts.join("input.3mf");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"),
        &project,
    )
    .unwrap();
    let native = artifacts.join("native");
    let ares = artifacts.join("ares");
    fs::create_dir(&native).unwrap();
    fs::create_dir(&ares).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            native.to_str().unwrap().to_owned(),
            project.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &artifacts,
    )
    .unwrap();
    let command = fs::read_dir(&artifacts)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("command-")
        })
        .unwrap();
    let native_bytes = fs::read(native.join("result.json")).unwrap();
    let reference: serde_json::Value = serde_json::from_slice(&native_bytes).unwrap();
    assert_eq!(reference["return_code"], 0, "Native must succeed");
    let warning = reference["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|warning| warning["class"] == "slicing_warning_non_critical")
        .expect("the actual Native run must witness the noncritical support warning");
    assert_eq!(warning["plate_id"], 1);
    assert!(!warning["text"].as_str().unwrap().is_empty());
    assert_eq!(
        reference["sliced_plates"][0]["warning_message"],
        warning["text"]
    );

    let output = crate::runner::application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(ares.join("plate_1.gcode"))
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("ares.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(0), "{artifacts:?}");
    assert_eq!(output.stdout, fs::read(command.join("stdout.bin")).unwrap());
    let native_stderr = fs::read(command.join("stderr.bin")).unwrap();
    assert!(
        native_stderr.is_empty(),
        "this witness has no stderr warning"
    );
    assert_eq!(output.stderr, native_stderr, "stderr: {artifacts:?}");
    let actual = fs::read(ares.join("result.json")).unwrap();
    let report: serde_json::Value = serde_json::from_slice(&actual).unwrap();
    assert_eq!(report["return_code"], 0, "success writer is a prerequisite");
    // Full reports include every warning and every plate warning_message. Only the
    // existing wall-clock normalization is used; no warning text is synthesized.
    assert_eq!(
        result_comparison_bytes(&actual),
        result_comparison_bytes(&native_bytes),
        "complete result.json including support warning: {artifacts:?}"
    );
}
