#![cfg(target_os = "linux")]

use std::{
    ffi::OsString,
    fs,
    io::{Cursor, Write},
    path::Path,
};

use serde_json::{Value, json};

use crate::{artifacts, native_wall_clock::result_comparison_bytes, runner::OrcaRunner};

use super::cli_no_check::{inventory, run};

const SUMMARY: &str = "Filaments are not compatible with the plate type. Please verify the slicing of all plates in Orca Slicer before uploading.";
const DETAIL: &str = "Plate 1: Cool Plate does not support filament 1\n";

#[test]
fn cli_rejects_a_filament_without_a_normal_temperature_for_the_selected_bed() {
    compare("0", &[], SUMMARY);
}

#[test]
fn cli_no_check_keeps_the_bed_filament_rejection_and_reports_its_detail() {
    compare("0", &["--no-check"], DETAIL);
}

#[test]
fn cli_accepts_a_filament_with_a_normal_temperature_for_the_selected_bed() {
    compare("45", &[], "");
}

fn compare(temperature: &str, actions: &[&str], expected_error: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-bed-filament-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("project.3mf");
    fs::write(&input, project(temperature)).unwrap();
    let mut argv: Vec<OsString> = ["--debug", "0", "--outputdir", "out"]
        .into_iter()
        .chain(actions.iter().copied())
        .chain(["--slice", "1"])
        .map(OsString::from)
        .collect();
    argv.push(input.into());
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let ares_report = fs::read(ares_cwd.join("out/result.json")).unwrap();
    let report: Value = serde_json::from_slice(&native_report).unwrap();
    if expected_error.is_empty() {
        assert_eq!(native.status.code(), Some(0), "{root:?}");
        assert_eq!(report["return_code"], 0, "{root:?}");
        assert!(native_cwd.join("out/plate_1.gcode").is_file(), "{root:?}");
        assert!(ares_cwd.join("out/plate_1.gcode").is_file(), "{root:?}");
    } else {
        assert_eq!(native.status.code(), Some(195), "{root:?}");
        assert_eq!(report["return_code"], -61, "{root:?}");
        assert_eq!(report["error_string"], expected_error, "{root:?}");
        assert_eq!(native.stderr, format!("{DETAIL}\n").as_bytes(), "{root:?}");
        assert_eq!(ares.stderr, native.stderr, "{root:?}");
        assert_eq!(
            inventory(&native_cwd),
            ["out/", "out/result.json"],
            "{root:?}"
        );
        assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
        assert!(
            result_comparison_bytes(&ares_report) == result_comparison_bytes(&native_report),
            "complete report differs: {root:?}"
        );
    }
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
}

fn project(temperature: &str) -> Vec<u8> {
    let base = include_bytes!("../../../../tests/parity/validation/e5max-scv/project.3mf");
    let mut source = zip::ZipArchive::new(Cursor::new(base)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let entry = source.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_reader(entry).unwrap();
            settings["cool_plate_temp"] = json!([temperature]);
            output
                .start_file(
                    "Metadata/project_settings.config",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}
