//! Orca 40ecc66f7: GCode.cpp:2404–2467 / OrcaSlicer.cpp:7141–7172.
use super::OrcaRunner;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext, WarningLevel};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

const WARNING: &str = "The object has empty layers between 0.4 and 1.8 and can’t be printed.\nObject: cube10.stl\nParts of the object at these heights may be too thin or the object may have a faulty mesh.";

macro_rules! case {
    ($name:ident, $input:literal) => {
        #[tokio::test(flavor = "current_thread")]
        async fn $name() {
            let Some(runner) = OrcaRunner::from_env().unwrap() else {
                crate::native_environment::skipped("mesh odd-order Native unavailable");
                return;
            };
            reject(
                &runner,
                $input,
                include_bytes!(concat!(
                    "../../../../../tests/parity/mesh-edge-pairing/",
                    $input,
                    ".3mf"
                )),
            )
            .await;
        }
    };
}
case!(odd_rotated_requires_empty_first_layer_error, "odd-rotated");
case!(odd_reverse_rejection_control, "odd-reverse");

async fn reject(runner: &OrcaRunner, label: &str, input: &[u8]) {
    let source = super::repo_root().join("OrcaSlicer/src");
    let utils = fs::read_to_string(source.join("libslic3r/Utils.hpp")).unwrap();
    let slicing_error: i32 = utils
        .lines()
        .find_map(|line| line.strip_prefix("#define CLI_SLICING_ERROR"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let cli_source = fs::read_to_string(source.join("OrcaSlicer.cpp")).unwrap();
    let slicing_message = cli_source
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("{CLI_SLICING_ERROR, \"")
                .and_then(|message| message.strip_suffix("\"},"))
        })
        .unwrap();
    let root = tempfile::Builder::new()
        .prefix("mesh-odd-order-admission-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = root.join("input.3mf");
    fs::write(&path, input).unwrap();
    let output = root.join("native");
    fs::create_dir(&output).unwrap();
    let native = super::stages::run(
        runner.bin(),
        [
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            output.to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        super::stages::Stage::Slice,
        &root,
    );
    let overrides = ConfigOverrides::default();
    let api = match super::application::project_supply().unwrap() {
        None => {
            ares_core::slice_project_plate_processed_validated(
                input,
                GenerationMetadata::default(),
                None,
                &overrides,
                SlicerContext::new(0),
            )
            .await
        }
        Some(supply) => {
            ares_core::slice_with_profiles_processed_validated(
                input,
                ares_core::ProfileSliceRequest {
                    filename: "project.3mf",
                    catalog_json: &supply.catalog,
                    selection_json: r#"{"from_project":true,"uptodate":false}"#,
                    plate: None,
                    overrides: &overrides,
                    placement: ares_core::Placement::None,
                },
                GenerationMetadata::default(),
                SlicerContext::new(0),
            )
            .await
        }
    };
    if let Ok(processed) = &api {
        fs::write(root.join("ares.gcode"), &processed.gcode).unwrap();
    }
    let mut cli_results = Vec::new();
    for (name, supplied, moves) in [
        ("cli", true, false),
        ("cli-plain", false, false),
        ("cli-dump", true, true),
        ("cli-plain-dump", false, true),
    ] {
        let directory = root.join(name);
        fs::create_dir(&directory).unwrap();
        let cli_path = directory.join("output.gcode");
        let moves_path = directory.join("moves.csv");
        let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
        command.arg("slice").arg("-o").arg(&cli_path);
        if supplied && let Some(supply) = super::application::project_supply().unwrap() {
            command
                .arg("--project-preset-catalog")
                .arg(&supply.catalog_path);
        }
        if moves {
            command.arg("--dump-moves").arg(&moves_path);
        }
        command.arg(&path);
        let cli = command.output().unwrap();
        fs::write(directory.join("stdout"), &cli.stdout).unwrap();
        fs::write(directory.join("stderr"), &cli.stderr).unwrap();
        cli_results.push(json!({
            "name":name, "exit":cli.status.code(),
            "gcode_exists":cli_path.exists(), "moves_exists":moves_path.exists(),
        }));
    }
    let native_result: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    let sha = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let api_warnings = api.as_ref().ok().map(|processed| {
        processed
            .warnings
            .iter()
            .map(|warning| {
                json!({
                    "level":format!("{:?}", warning.level),
                    "message_type":warning.message_type as u8,
                    "message":warning.message,
                })
            })
            .collect::<Vec<_>>()
    });
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "classification":"warning-adapter-admission-parity",
            "label":label, "input_sha256":sha(input),
            "native_result":native_result, "native_stage":format!("{native:?}"),
            "api":api.as_ref().map(|processed| sha(&processed.gcode)).map_err(ToString::to_string),
            "api_warnings":api_warnings, "cli":cli_results,
        }))
        .unwrap(),
    )
    .unwrap();
    let error = native.unwrap_err();
    assert_eq!(error.kind, super::stages::FailureKind::Process, "{root:?}");
    assert_eq!(native_result["return_code"], slicing_error, "{root:?}");
    let captured = error.captured.unwrap();
    let exit_code = Some(if cfg!(unix) {
        slicing_error.rem_euclid(256)
    } else {
        slicing_error
    });
    assert_eq!(captured.code, exit_code, "{root:?}");
    assert_eq!(native_result["error_string"], slicing_message, "{root:?}");
    assert_eq!(
        native_result["sliced_plates"][0]["warning_message"], WARNING,
        "{root:?}"
    );
    assert!(!output.join("plate_1.gcode").exists(), "{root:?}");
    let processed = api.unwrap_or_else(|error| {
        panic!("{label}: core must retain the GUI warning, not reject: {error}; {root:?}")
    });
    assert!(!processed.gcode.is_empty(), "{root:?}");
    let warnings = processed
        .warnings
        .iter()
        .filter(|warning| warning.message_type as u8 == 3)
        .collect::<Vec<_>>();
    assert_eq!(
        warnings.len(),
        1,
        "missing SlicingEmptyGcodeLayers; {root:?}"
    );
    assert_eq!(warnings[0].level, WarningLevel::Critical, "{root:?}");
    assert_eq!(warnings[0].message, WARNING, "{root:?}");
    for result in cli_results {
        let name = result["name"].as_str().unwrap();
        assert_eq!(result["exit"], json!(exit_code), "{name}; {root:?}");
        assert_eq!(result["gcode_exists"], false, "{name}; {root:?}");
        assert_eq!(result["moves_exists"], false, "{name}; {root:?}");
        #[cfg(target_os = "linux")]
        {
            let result: serde_json::Value =
                serde_json::from_slice(&fs::read(root.join(name).join("result.json")).unwrap())
                    .unwrap();
            assert_eq!(result["return_code"], slicing_error, "{name}; {root:?}");
            assert_eq!(result["error_string"], slicing_message, "{name}; {root:?}");
            assert_eq!(
                result["sliced_plates"][0]["warning_message"], WARNING,
                "{name}; {root:?}"
            );
            assert_eq!(
                result["sliced_plates"][0]["triangle_count"], 0,
                "{name}; {root:?}"
            );
        }
    }
}
