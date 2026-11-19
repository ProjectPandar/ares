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
    run_case(true, 1).await;
}

#[tokio::test]
async fn two_loaded_slots_keep_native_flush_matrix_rejection() {
    run_case(true, 2).await;
}

#[tokio::test]
async fn embedded_filament_selection_public_whole_output_control() {
    run_case(false, 1).await;
}

async fn run_case(empty: bool, count: usize) {
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
    let mut filament_paths = vec![filament.clone()];
    if count == 2 {
        let mut second: Value = serde_json::from_slice(&fs::read(&filament).unwrap()).unwrap();
        second["name"] = json!(format!(
            "{} - second slot",
            second["name"].as_str().unwrap()
        ));
        let path = root.join("second-filament.json");
        fs::write(&path, serde_json::to_vec_pretty(&second).unwrap()).unwrap();
        filament_paths.push(path);
    }
    let filament_args = filament_paths
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect::<Vec<_>>()
        .join(";");
    let setting_paths = format!("{};{}", machine.display(), process.display());
    let native_result = crate::runner::stages::run(
        runner.bin(),
        [
            "--slice",
            "0",
            "--load-settings",
            &setting_paths,
            "--load-filaments",
            &filament_args,
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            input_path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        crate::runner::stages::Stage::Slice,
        &root,
    );
    let native_path = root.join("native-output/plate_1.gcode");
    let matrix_error = "Flush volumes matrix do not match to the correct size!";
    let native = if count == 2 {
        let failure = native_result.unwrap_err();
        assert_eq!(failure.kind, crate::runner::stages::FailureKind::Process);
        let evidence = failure.evidence.unwrap();
        let status: Value =
            serde_json::from_slice(&fs::read(evidence.join("status.json")).unwrap()).unwrap();
        assert_eq!(status["code"], 156);
        assert_eq!(
            fs::read(evidence.join("stdout.bin")).unwrap(),
            b"run found error, return -100, exit...\n"
        );
        assert!(
            String::from_utf8_lossy(&fs::read(evidence.join("stderr.bin")).unwrap())
                .contains(matrix_error)
        );
        assert!(!native_path.exists());
        None
    } else {
        native_result.unwrap();
        Some(fs::read(native_path).unwrap())
    };
    let machine_json = fs::read_to_string(&machine).unwrap();
    let process_json = fs::read_to_string(&process).unwrap();
    let filament_jsons = filament_paths
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();
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
    let filaments = filament_paths
        .iter()
        .zip(&filament_jsons)
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
        })
        .collect::<Vec<_>>();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let public = ares_core::slice_with_load_settings(
        &input,
        LoadSettingsSliceRequest {
            transforms: &ares_core::CliProjectTransforms::NONE,
            filename: "input.3mf",
            catalog_json: None,
            uptodate: false,
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
    let comparison = if let Some(native) = &native {
        public
            .as_ref()
            .map_err(|error| error.to_string())
            .and_then(|gcode| {
                let expected = crate::load_settings_cli::banner_normalized(native);
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
            })
    } else {
        match &public {
            Err(error) if error.to_string().contains(matrix_error) => Ok(()),
            Err(error) => Err(format!("Native matrix rejection differs: {error}")),
            Ok(_) => Err("API accepted the Native matrix rejection".to_owned()),
        }
    };
    if let Ok(gcode) = &public {
        fs::write(root.join("api.gcode"), gcode).unwrap();
    }
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "original_sha256": digest(&original), "input_sha256": digest(&input),
        "machine_sha256": digest(machine_json.as_bytes()), "process_sha256": digest(process_json.as_bytes()), "filament_sha256": filament_jsons.iter().map(|json| digest(json.as_bytes())).collect::<Vec<_>>(),
        "native_sha256": native.as_ref().map(|bytes| digest(bytes)), "api_sha256": public.as_ref().ok().map(|bytes| digest(bytes)),
        "api_error": public.as_ref().err().map(ToString::to_string), "whole_output": comparison,
        "input_change": if empty { "filament_settings_id becomes [] only" } else { "none" }, "loaded_filament_count": count,
    })).unwrap()).unwrap();
    assert!(
        comparison.is_ok(),
        "complete Native/API output differs: {comparison:?}; artifacts {}",
        root.display()
    );
}
