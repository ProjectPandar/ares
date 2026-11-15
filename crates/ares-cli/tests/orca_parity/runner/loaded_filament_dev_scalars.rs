#![cfg(target_os = "linux")]

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SliceError, SlicedPlate, SlicerContext,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

use super::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
const KEY: &str = "filament_dev_ams_drying_ams_limitations";

fn preset(value: Value) -> String {
    json!({
        "type": "filament", "from": "system", "name": "Device wire",
        "instantiation": "true", KEY: value,
    })
    .to_string()
}

fn settings() -> Vec<String> {
    let profiles = crate::presets::VendorProfiles::load(
        &super::repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    [
        (
            "machine",
            profiles.machine("Bambu Lab X1 Carbon 0.4 nozzle").unwrap(),
        ),
        (
            "process",
            profiles.process("0.20mm Standard @BBL X1C").unwrap(),
        ),
    ]
    .into_iter()
    .map(|(kind, mut fields)| {
        fields.remove("inherits");
        fields.remove("include");
        fields.insert("type".to_owned(), json!(kind));
        fields.insert("from".to_owned(), json!("system"));
        fields.insert("instantiation".to_owned(), json!("true"));
        Value::Object(fields).to_string()
    })
    .collect()
}

fn write_settings(directory: &Path, settings: &[String]) {
    fs::write(directory.join("machine.json"), &settings[0]).unwrap();
    fs::write(directory.join("process.json"), &settings[1]).unwrap();
}

async fn public_slice(json: &str, settings: &[String]) -> Result<Vec<u8>, SliceError> {
    let files = settings
        .iter()
        .map(|json| LoadSettingsFile {
            label: "settings.json",
            json,
        })
        .collect::<Vec<_>>();
    ares_core::slice_with_load_settings(
        INPUT,
        LoadSettingsSliceRequest {
            filename: "input.3mf",
            catalog_json: None,
            uptodate: false,
            settings: &files,
            filaments: &[LoadSettingsFile {
                label: "filament.json",
                json,
            }],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            orient: false,
            sliced_plate: &mut SlicedPlate::default(),
        },
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
}

fn native(
    runner: &OrcaRunner,
    directory: &Path,
    filename: &str,
) -> Result<Vec<u8>, stages::StageError> {
    let output = directory.join(format!("native-{filename}"));
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--load-settings".to_owned(),
            format!(
                "{};{}",
                directory.join("machine.json").display(),
                directory.join("process.json").display()
            ),
            "--load-filaments".to_owned(),
            directory.join(filename).to_str().unwrap().to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            directory.join("input.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        directory,
    )?;
    Ok(fs::read(output.join("plate_1.gcode")).unwrap())
}

#[tokio::test]
async fn scalar_device_wire_matches_vector_and_rejects_invalid_wire() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("filament-device-scalars-")
        .tempdir()
        .unwrap()
        .keep();
    let settings = settings();
    // Config.cpp:985-987 and Config.hpp:1225-1229 accept coStrings scalar wire,
    // including its empty-vector spelling; malformed quoted wire fails decoding.
    for (name, vector, scalar) in [
        ("nonempty", json!(["1", "0"]), json!("1;0")),
        ("empty", json!([]), json!("")),
    ] {
        let directory = root.join(name);
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("input.3mf"), INPUT).unwrap();
        write_settings(&directory, &settings);
        let vector = preset(vector);
        let scalar = preset(scalar);
        fs::write(directory.join("vector.json"), &vector).unwrap();
        fs::write(directory.join("scalar.json"), &scalar).unwrap();
        let native_vector = native(&runner, &directory, "vector.json").unwrap();
        let native_scalar = native(&runner, &directory, "scalar.json").unwrap();
        assert_eq!(
            crate::load_settings_cli::banner_normalized(&native_scalar),
            crate::load_settings_cli::banner_normalized(&native_vector)
        );
        let expected = public_slice(&vector, &settings).await.unwrap();
        fs::write(directory.join("ares-vector.gcode"), &expected).unwrap();
        let actual = public_slice(&scalar, &settings).await.unwrap();
        fs::write(directory.join("ares-scalar.gcode"), &actual).unwrap();
        assert_eq!(actual, expected);
    }
    let directory = root.join("invalid");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("input.3mf"), INPUT).unwrap();
    write_settings(&directory, &settings);
    let invalid = preset(json!("\"unterminated"));
    fs::write(directory.join("invalid.json"), &invalid).unwrap();
    let error = native(&runner, &directory, "invalid.json").unwrap_err();
    assert_eq!(error.kind, stages::FailureKind::Process);
    let error = public_slice(&invalid, &settings).await.unwrap_err();
    fs::write(directory.join("ares-error.txt"), error.to_string()).unwrap();
    assert!(matches!(error, SliceError::InvalidInput(message) if message.contains(KEY)));
}
