//! Config.hpp:1083; PlaceholderParser.cpp:1118-1119,1352-1353.
use crate::runner::{OrcaRunner, application, native_stream_case, stages};
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const INPUT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/pre-cooling-variant/mixed.3mf"
));
const NIL_ERROR: &str =
    "Trying to reference an undefined (nil) element of vector of optional values";

fn reference_input(template: &str) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["filament_pre_cooling_temperature"] = json!(["nil", "nil", "123", "0"]);
            settings["machine_start_gcode"] = json!(template);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        target
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        target.write_all(&bytes).unwrap();
    }
    target.finish().unwrap().into_inner()
}

#[tokio::test(flavor = "current_thread")]
async fn nullable_vector_unreferenced_nil_does_not_panic() {
    let (reference, _) = native_stream_case::qualify(
        (
            "nullable-vector-placeholder",
            &["Config.hpp:1083-1135", "PlaceholderParser.cpp:1118-1119"],
        ),
        "unreferenced",
        INPUT,
        "561ca738f30133e99af9b162ceb2e83586d1068edfa09b2e5ddf2d914d6ca387",
        false,
    );
    for line in [
        "; filament_pre_cooling_temperature = 0,nil",
        "; filament_pre_cooling_temperature_nc = nil,0",
    ] {
        assert!(reference.native.lines().any(|actual| actual == line));
    }
    let api = application::slice_project_plate_validated(
        INPUT,
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    fs::write(
        reference.root.join("api-result.json"),
        serde_json::to_vec_pretty(
            &api.as_ref()
                .map(|bytes| bytes.len())
                .map_err(ToString::to_string),
        )
        .unwrap(),
    )
    .unwrap();
    let api = api.unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let output = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success());
    assert_eq!(api, fs::read(output).unwrap());
}

#[tokio::test(flavor = "current_thread")]
async fn nullable_vector_nil_reference_returns_slice_error() {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    for (label, template) in [
        ("brace", "; NIL_REF {filament_pre_cooling_temperature[0]}\n"),
        ("legacy", "; NIL_REF [filament_pre_cooling_temperature_0]\n"),
    ] {
        let input = reference_input(template);
        let root = tempfile::Builder::new()
            .prefix(&format!("nullable-reference-{label}-"))
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let path = root.join("input.3mf");
        fs::write(&path, &input).unwrap();
        let output = root.join("native");
        fs::create_dir(&output).unwrap();
        let native = stages::run(
            runner.bin(),
            [
                "--slice".into(),
                "0".into(),
                "--outputdir".into(),
                output.to_string_lossy().into_owned(),
                path.to_string_lossy().into_owned(),
            ],
            stages::Stage::Slice,
            &root,
        );
        let native_result = fs::read_to_string(output.join("result.json")).unwrap();
        assert!(
            native.is_err(),
            "Native must reject nil reference: {root:?}"
        );
        let native_stderr = &native
            .as_ref()
            .unwrap_err()
            .captured
            .as_ref()
            .unwrap()
            .stderr;
        assert!(
            String::from_utf8_lossy(native_stderr).contains(NIL_ERROR),
            "{native_result}"
        );
        let api = application::slice_project_plate_validated(
            &input,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        )
        .await;
        let cli = application::cli_command()
            .args(["slice", "-o"])
            .arg(root.join("ares.cli.gcode"))
            .arg(&path)
            .output()
            .unwrap();
        fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
        fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({"input_sha256": Sha256::digest(&input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),"api":api.as_ref().map(|b|b.len()).map_err(ToString::to_string),"cli_exit":cli.status.code(),"native_result":native_result})).unwrap()).unwrap();
        assert!(
            matches!(api, Err(SliceError::CustomGCode { ref message, .. }) if message.contains(NIL_ERROR)),
            "{api:?}"
        );
        assert!(!cli.status.success());
        assert!(String::from_utf8_lossy(&cli.stderr).contains(NIL_ERROR));
    }
}
