//! M854 oracle: user option overrides (`ConfigOverrides`) must slice
//! byte-identically to official OrcaSlicer consuming the same values written
//! into `Metadata/project_settings.config` — the upstream GUI's
//! `PresetBundle::full_config` → `Print::apply` path
//! (`libslic3r/Config.cpp:573-620 set_deserialize`, `PrintApply.cpp:1113`).

use std::io::{Cursor, Read, Write};

use serde_json::Value;
use serde_json::json;

use super::{ExportedCase, OrcaRunner};

fn overrides() -> Vec<(&'static str, Value)> {
    vec![
        ("layer_height", json!("0.3")),
        ("wall_loops", json!("4")),
        ("sparse_infill_density", json!("30%")),
        ("enable_support", json!("1")),
        // Vector overrides use the upstream `serialize()` comma token
        // (inverse of `decodeConfigValue`); no per-index writes.
        ("nozzle_temperature", json!("210,220")),
    ]
}

/// Same values in `project_settings.config` document shape (string or
/// array of strings) for the rewritten-3MF oracle side.
fn document_edits() -> Vec<(&'static str, Value)> {
    overrides()
        .into_iter()
        .map(|(key, value)| match value {
            Value::String(text) if key == "nozzle_temperature" => (
                key,
                Value::Array(
                    text.split(',')
                        .map(|item| Value::String(item.to_owned()))
                        .collect(),
                ),
            ),
            other => (key, other),
        })
        .collect()
}

fn overrides_json() -> String {
    let mut object = serde_json::Map::new();
    for (key, value) in overrides() {
        object.insert(key.to_owned(), value);
    }
    serde_json::to_string(&object).unwrap()
}

/// Rewrites `Metadata/project_settings.config` in a copy of the project,
/// keeping every other archive entry byte-identical.
fn with_project_settings(project: &[u8], edits: &[(&str, Value)]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            let settings = settings.as_object_mut().unwrap();
            for (key, value) in edits {
                settings.insert((*key).to_owned(), value.clone());
            }
            bytes = serde_json::to_vec(settings).unwrap();
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

#[test]
fn user_overrides_match_documented_project_settings_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_OVERRIDE_REFERENCE") {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        std::fs::read(super::repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let edits = document_edits();
    let edited = with_project_settings(&project, &edits);

    // Official OrcaSlicer slices the copy carrying the edited values.
    let project_path = runner.work.join("m854-user-overrides.3mf");
    let output_dir = runner.work.join("m854-user-overrides-orca");
    std::fs::write(&project_path, &edited).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: "user-overrides/ender3".into(),
            project: edited.clone(),
            project_path,
            output_dir,
        })
        .unwrap();

    // Ares slices the original document with the same values as overrides.
    let overrides = ares_core::ConfigOverrides::from_json(overrides_json().as_bytes()).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &overrides,
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();

    let outcome = crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &actual);
    assert!(
        outcome.is_ok(),
        "overridden slice diverged from documented project_settings output: {outcome:?}"
    );
}
