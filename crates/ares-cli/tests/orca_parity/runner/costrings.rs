//! ConfigOptionStrings import/export: PrintConfig.cpp:4193, Config.cpp:72-120.
use std::{
    fs,
    io::{Cursor, Read as _, Write as _},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

const INPUT: &[u8] = include_bytes!("../../../../../tests/parity/validation/project-array.3mf");

#[tokio::test]
async fn costrings_embedded_inherits_group_matches_native_complete_output() {
    compare("unchanged-project-array", INPUT).await;
}

#[tokio::test]
async fn costrings_embedded_arrays_escape_logical_slots_without_changing_typed_text() {
    let settings = json!({
        "preset_names": ["space tab\tbackslash\\quote\"", "line\nfeed", "carriage\rreturn"],
        "filament_notes": ["logical space\ttab\\slash\"quote\nline\rcarriage;semicolon"]
    });
    compare("array-escaping-and-typed-text", &with_settings(&settings)).await;
}

#[tokio::test]
async fn costrings_embedded_empty_slot_shapes_match_native_complete_output() {
    for (label, slots) in [
        ("singleton-empty", json!([""])),
        ("multi-empty", json!(["", ""])),
        ("empty-vector", json!([])),
    ] {
        compare(label, &with_settings(&json!({"preset_names": slots}))).await;
    }
}

#[tokio::test]
async fn costrings_embedded_scalar_wire_decodes_to_logical_slots() {
    let settings = json!({"preset_names": "\"space tab\tbackslash\\\\quote\\\"\";\"line\\nfeed\";\"carriage\\rreturn;semicolon\""});
    compare("scalar-wire", &with_settings(&settings)).await;
}

fn with_settings(changes: &serde_json::Value) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in changes.as_object().unwrap() {
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
    output.finish().unwrap().into_inner()
}

async fn compare(label: &str, input: &[u8]) {
    if crate::native_environment::binary("ARES_ORCA_BIN").is_none() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("Native runner unavailable for coStrings comparison");
        return;
    };
    let mut builder = tempfile::Builder::new();
    builder.prefix("costrings-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, input).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let reference = runner
        .rerun_reference(&ExportedCase {
            label: format!("costrings/{label}"),
            project: input.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap()
        .reference;
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let actual = crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &actual {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    let hash = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({"case": label, "input_sha256": hash, "native_bytes": reference.len(), "ares": actual.as_ref().map(Vec::len).map_err(ToString::to_string)})).unwrap()).unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
}
