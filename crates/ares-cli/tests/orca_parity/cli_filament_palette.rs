//! Support/palette admission: OrcaSlicer.cpp:3929–3933.

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::Command,
};

use serde_json::{Value, json};

use crate::{native_wall_clock::result_comparison_bytes, runner::EarlyExit};

#[test]
fn cli_filament_support_palette_mismatch_matches_native() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("cli-filament-support-palette-")
        .tempdir()
        .unwrap()
        .keep();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/classic-layer-id/m-later-owner.3mf");
    let project = artifacts.join("filament-support-palette-mismatch.3mf");
    fs::write(&project, with_two_colors(&fs::read(source).unwrap())).unwrap();
    let native = runner
        .early_cli_reference(&project, EarlyExit::FilamentSupportPaletteMismatch)
        .unwrap();
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    let receipt = fs::read(native.directory.join("receipt.json")).unwrap();
    fs::write(artifacts.join("native.receipt.json"), &receipt).unwrap();
    let receipt: Value = serde_json::from_slice(&receipt).unwrap();
    let native_exit = receipt["exit"].as_i64().unwrap() as i32;
    let native_report: Value = serde_json::from_slice(&native.result_json).unwrap();
    assert_eq!(native_report["return_code"], -5);
    assert_eq!(native_exit, 251);
    assert!(native.gcode.is_none());

    let gcode = artifacts.join("plate_1.gcode");
    let overrides = json!({
        "filament_colour": ["#FFFFFF", "#000000"],
        "filament_is_support": ["0"]
    })
    .to_string();
    // Use the direct project route with the reference's system preset resources.
    let resources = crate::native_environment::value("ARES_ORCA_RESOURCES_DIR")
        .expect("Native reference resources");
    let argv = vec![
        "slice".to_owned(),
        "--resources-dir".to_owned(),
        resources.to_string_lossy().into_owned(),
        project.to_string_lossy().into_owned(),
        "-o".to_owned(),
        gcode.to_string_lossy().into_owned(),
        "--overrides".to_owned(),
        overrides,
    ];
    fs::write(
        artifacts.join("ares.argv.json"),
        serde_json::to_vec_pretty(&argv).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(&argv)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit.json"),
        json!({"code": output.status.code()}).to_string(),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(native_exit), "{artifacts:?}");
    assert_eq!(output.stdout, native.stdout, "stdout: {artifacts:?}");
    assert_eq!(output.stderr, native.stderr, "stderr: {artifacts:?}");
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json),
        "complete result.json: {artifacts:?}"
    );
    assert_eq!(gcode.exists(), native.gcode.is_some(), "{artifacts:?}");
}

fn with_two_colors(input: &[u8]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut config: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(config["filament_colour"].as_array().unwrap().len(), 1);
            assert_eq!(config["filament_is_support"], json!(["0"]));
            config["filament_colour"] = json!(["#FFFFFF", "#000000"]);
            bytes = serde_json::to_vec(&config).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
