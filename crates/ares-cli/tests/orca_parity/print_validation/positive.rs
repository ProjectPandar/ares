use std::io::{Cursor, Read, Write};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};

use super::{RELATIVE_INPUT, sha256};
use crate::runner::{OrcaRunner, stages};

#[test]
fn marlin_relative_accepts_active_reset_in_before_template() {
    verify_positive_override(
        "marlin-with-reset",
        json!({"before_layer_change_gcode": "G92 E0\n"}),
    );
}

#[test]
fn non_marlin_relative_does_not_require_layer_reset() {
    verify_positive_override("klipper-without-reset", json!({"gcode_flavor": "klipper"}));
}

#[test]
fn bbl_marlin_relative_does_not_require_layer_reset() {
    verify_positive_override(
        "bbl-without-reset",
        json!({"printer_model": "Bambu Lab P1S"}),
    );
}

fn verify_positive_override(label: &str, edits: Value) {
    let overrides = ConfigOverrides::from_json(&serde_json::to_vec(&edits).unwrap()).unwrap();
    let directory = tempfile::Builder::new()
        .prefix("o1-positive-")
        .tempdir()
        .unwrap()
        .keep();
    let mut input = zip::ZipArchive::new(Cursor::new(RELATIVE_INPUT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in edits.as_object().unwrap() {
                settings[key] = value.clone();
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let edited = output.finish().unwrap().into_inner();
    let path = directory.join("input.3mf");
    std::fs::write(&path, &edited).unwrap();
    let oracle = OrcaRunner::from_env().unwrap().map(|runner| {
        let output = directory.join("orca");
        std::fs::create_dir(&output).unwrap();
        stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap();
        let gcode = std::fs::read(output.join("plate_1.gcode")).unwrap();
        assert!(!gcode.is_empty());
        json!({"status": "GCODE", "sha256": sha256(&gcode)})
    });
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            RELATIVE_INPUT,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &overrides,
            SlicerContext::new(0),
        ));
    let outcome = match &actual {
        Ok(gcode) => {
            std::fs::write(directory.join("ares.gcode"), gcode).unwrap();
            json!({"status": "GCODE", "sha256": sha256(gcode)})
        }
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    std::fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label,
            "input_sha256": sha256(&edited),
            "overrides": edits,
            "oracle": oracle,
            "ares": outcome,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        actual.is_ok(),
        "valid upstream gate case must not be rejected: {}",
        directory.display()
    );
    assert!(!actual.unwrap().is_empty());
}
