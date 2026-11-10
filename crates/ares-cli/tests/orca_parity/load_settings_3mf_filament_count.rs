use std::{
    fs,
    io::{Cursor, Read, Write},
};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SlicerContext,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::load_settings_3mf_fixture;

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn without_selection(input: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(
                settings["filament_settings_id"],
                json!(["Prusa Generic PLA"])
            );
            settings["filament_settings_id"] = json!([]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        writer
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[tokio::test]
async fn zero_embedded_filament_selection_uses_loaded_count_whole_output() {
    run_case(true).await;
}

#[tokio::test]
async fn embedded_filament_selection_public_whole_output_control() {
    run_case(false).await;
}

async fn run_case(empty: bool) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("3mf-loaded-filament-count-")
        .tempdir()
        .unwrap()
        .keep();
    let original = fs::read(load_settings_3mf_fixture("compatible-input.3mf")).unwrap();
    let input = if empty {
        without_selection(&original)
    } else {
        original.clone()
    };
    let input_path = root.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    let machine = load_settings_3mf_fixture("machine.json");
    let process = load_settings_3mf_fixture("process.json");
    let filament = load_settings_3mf_fixture("compatible-filament-0.json");
    let setting_paths = format!("{};{}", machine.display(), process.display());
    crate::load_settings_cli::stages_run(
        &runner,
        &root,
        &[
            "--slice",
            "0",
            "--load-settings",
            &setting_paths,
            "--load-filaments",
            filament.to_str().unwrap(),
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            input_path.to_str().unwrap(),
        ],
    );
    let native = fs::read(root.join("native-output/plate_1.gcode")).unwrap();
    let machine_json = fs::read_to_string(&machine).unwrap();
    let process_json = fs::read_to_string(&process).unwrap();
    let filament_json = fs::read_to_string(&filament).unwrap();
    let settings = [
        LoadSettingsFile {
            label: machine.to_str().unwrap(),
            json: &machine_json,
        },
        LoadSettingsFile {
            label: process.to_str().unwrap(),
            json: &process_json,
        },
    ];
    let filaments = [LoadSettingsFile {
        label: filament.to_str().unwrap(),
        json: &filament_json,
    }];
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let public = ares_core::slice_with_load_settings(
        &input,
        LoadSettingsSliceRequest {
            filename: "input.3mf",
            settings: &settings,
            filaments: &filaments,
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            sliced_plate: &mut sliced_plate,
        },
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await;
    let comparison = public
        .as_ref()
        .map_err(|error| error.to_string())
        .and_then(|gcode| {
            let expected = crate::load_settings_cli::banner_normalized(&native);
            let actual = crate::load_settings_cli::banner_normalized(gcode);
            if expected == actual {
                Ok(())
            } else {
                let at = expected
                    .iter()
                    .zip(&actual)
                    .position(|(a, b)| a != b)
                    .unwrap_or(expected.len().min(actual.len()));
                Err(format!(
                    "first difference at byte {at}; Native {} bytes, API {} bytes",
                    expected.len(),
                    actual.len()
                ))
            }
        });
    if let Ok(gcode) = &public {
        fs::write(root.join("api.gcode"), gcode).unwrap();
    }
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "original_sha256": digest(&original), "input_sha256": digest(&input),
        "machine_sha256": digest(machine_json.as_bytes()), "process_sha256": digest(process_json.as_bytes()), "filament_sha256": digest(filament_json.as_bytes()),
        "native_sha256": digest(&native), "api_sha256": public.as_ref().ok().map(|bytes| digest(bytes)),
        "api_error": public.as_ref().err().map(ToString::to_string), "whole_output": comparison,
        "input_change": if empty { "filament_settings_id becomes [] only" } else { "none" }, "loaded_filament_count": 1,
    })).unwrap()).unwrap();
    assert!(
        comparison.is_ok(),
        "complete Native/API output differs: {comparison:?}; artifacts {}",
        root.display()
    );
}
