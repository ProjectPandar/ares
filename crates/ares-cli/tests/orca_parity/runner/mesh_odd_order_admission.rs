//! GCode.cpp:1771-1778 / Layer.hpp:101,207,295; SPEC5b7cb9878.
use super::OrcaRunner;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

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
    let api = crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    let cli_path = root.join("cli.gcode");
    let cli = crate::runner::application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(&cli_path)
        .arg(&path)
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    if let Ok(bytes) = &api {
        fs::write(root.join("ares.gcode"), bytes).unwrap();
    }
    let native_result: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    let sha = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "classification":"admission-parity",
            "label":label, "input_sha256":sha(input),
            "native_result":native_result, "native_stage":format!("{native:?}"),
            "api":api.as_ref().map(|bytes| sha(bytes)).map_err(ToString::to_string),
            "api_debug":format!("{:?}", api.as_ref().err()),
            "cli_exit":cli.status.code(), "cli_gcode_exists":cli_path.exists(),
            "future_gap": api.is_ok() || cli.status.success(),
        }))
        .unwrap(),
    )
    .unwrap();
    let error = native.unwrap_err();
    assert_eq!(error.kind, super::stages::FailureKind::Process, "{root:?}");
    assert_eq!(native_result["return_code"], -100, "{root:?}");
    let captured = error.captured.unwrap();
    assert_eq!(
        captured.code,
        Some(if cfg!(unix) { 156 } else { -100 }),
        "{root:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&captured.stderr).trim(),
        "Errors",
        "{root:?}"
    );
    assert!(!output.join("plate_1.gcode").exists(), "{root:?}");
    let expected = ares_core::SliceError::InvalidInput(
        "one object has an empty first layer and cannot be printed".into(),
    );
    let error = api.err().unwrap_or_else(|| {
        panic!("{label}: Ares accepted Native empty-first-layer rejection; artifacts {root:?}")
    });
    assert_eq!(error, expected, "{root:?}");
    assert_eq!(cli.status.code(), Some(1), "{root:?}");
    assert!(
        String::from_utf8_lossy(&cli.stderr)
            .contains("one object has an empty first layer and cannot be printed"),
        "{root:?}"
    );
    assert!(!cli_path.exists(), "{root:?}");
}
