#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use super::plater_archive::rewrite_archive;
use crate::{artifacts, runner::OrcaRunner};

const SINGLE: &[u8] = include_bytes!("../../../../../../tests/parity/orca_cli_ender3.3mf");
const DOUBLE: &[u8] = include_bytes!(
    "../../../../../../tests/parity/validation/skirt_wipe_tower_flush/toolchange-wipe-flush.3mf"
);
const PROJECT: &str = "Metadata/project_settings.config";

#[test]
fn saved_export_settings_builds_missing_single_slot_flush_matrix() {
    compare("single-slot", SINGLE, 1, None);
}

#[test]
fn saved_export_settings_computes_missing_two_slot_flush_matrix() {
    compare("two-slots", DOUBLE, 2, None);
}

#[test]
fn saved_export_settings_retains_explicit_flush_matrix() {
    compare(
        "explicit-matrix",
        DOUBLE,
        2,
        Some(json!(["0", "83", "191", "0"])),
    );
}

fn compare(case: &str, archive: &[u8], slots: usize, supplied: Option<Value>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("saved-export-flush-{case}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = root.join("input.3mf");
    let input = rewrite_archive(archive, |name, text| {
        (name == PROJECT).then(|| {
            let mut config: Value = serde_json::from_str(text).unwrap();
            if let Some(matrix) = &supplied {
                config["flush_volumes_matrix"] = matrix.clone();
            } else {
                assert!(
                    config
                        .as_object_mut()
                        .unwrap()
                        .remove("flush_volumes_matrix")
                        .is_some()
                );
            }
            serde_json::to_string(&config).unwrap()
        })
    });
    fs::write(&project, input).unwrap();

    let args = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--export-settings",
        "out/settings.json",
        project.to_str().unwrap(),
    ];
    fs::write(root.join("argv.json"), serde_json::to_vec(&args).unwrap()).unwrap();
    let export = |bin: &Path, engine: &str| {
        let work = root.join(engine);
        fs::create_dir_all(work.join("out")).unwrap();
        let tmp = work.join("tmp");
        fs::create_dir(&tmp).unwrap();
        let output = Command::new(bin)
            .current_dir(&work)
            .env("TMPDIR", tmp)
            .env("ORCA_250_JOB_APPDIR", work.join("app"))
            .env("ORCA_JOB_APPDIR", work.join("app"))
            .args(args)
            .output()
            .unwrap();
        fs::write(work.join("stdout"), output.stdout).unwrap();
        fs::write(work.join("stderr"), output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(0), "{work:?}");
        serde_json::from_slice::<Value>(&fs::read(work.join("out/settings.json")).unwrap()).unwrap()
    };
    let native = export(runner.bin(), "native");
    let ares = export(Path::new(env!("CARGO_BIN_EXE_ares")), "ares");
    assert_eq!(
        native["filament_colour"].as_array().unwrap().len(),
        slots,
        "{root:?}"
    );
    let matrix = &native["flush_volumes_matrix"];
    assert_eq!(matrix.as_array().unwrap().len(), slots * slots, "{root:?}");
    if let Some(supplied) = supplied {
        assert_eq!(
            *matrix, supplied,
            "Native must retain the supplied matrix: {root:?}"
        );
    } else if slots == 2 {
        assert_ne!(matrix[1], "0", "{root:?}");
        assert_ne!(matrix[2], "0", "{root:?}");
    }
    for key in [
        "flush_volumes_matrix",
        "flush_volumes_vector",
        "flush_multiplier",
        "filament_colour",
        "filament_nozzle_map",
    ] {
        assert_eq!(ares[key], native[key], "{key}: {root:?}");
    }
}
