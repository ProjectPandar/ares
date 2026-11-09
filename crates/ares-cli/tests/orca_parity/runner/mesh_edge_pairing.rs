//! TriangleMesh.cpp:520-613; TriangleMeshSlicer.cpp:1865-1889.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

macro_rules! case {
    ($name:ident, $input:literal) => {
        #[tokio::test(flavor = "current_thread")]
        async fn $name() {
            let Some(runner) = OrcaRunner::from_env().unwrap() else {
                crate::native_environment::skipped("mesh-edge Native unavailable");
                return;
            };
            compare(
                &runner,
                stringify!($name),
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

case!(shared_original, "shared-original");
case!(split_original, "split-original");
case!(shared_reverse, "shared-reverse");
case!(shared_interleaved, "shared-interleaved");
case!(shared_rotated, "shared-rotated");
case!(cube_control, "cube-control");
case!(odd_original, "odd-original");
case!(six_original, "six-original");
case!(six_reverse, "six-reverse");
case!(six_rotated, "six-rotated");
case!(arachne_shared, "arachne-shared");
case!(arachne_split, "arachne-split");
case!(same_direction, "same-direction");

async fn compare(runner: &OrcaRunner, label: &str, input: &[u8]) {
    let mut builder = tempfile::Builder::new();
    builder.prefix("mesh-edge-pairing-");
    let root = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let path = root.join("input.3mf");
    fs::write(&path, input).unwrap();
    let native_output = root.join("native");
    fs::create_dir(&native_output).unwrap();
    let native = runner
        .rerun_reference(&ExportedCase {
            label: format!("mesh-edge-pairing/{label}"),
            project: input.to_vec(),
            project_path: path.clone(),
            output_dir: native_output,
        })
        .unwrap()
        .reference;
    fs::write(root.join("orca.gcode"), &native).unwrap();
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
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    if let Ok(bytes) = &api {
        fs::write(root.join("ares.gcode"), bytes).unwrap();
    }
    let difference = api.as_ref().ok().and_then(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(&native, bytes).err()
    });
    let sha = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "classification":"strict-parity", "label":label, "input_sha256":sha(input), "native_sha256":sha(&native),
            "api":api.as_ref().map(|bytes| sha(bytes)).map_err(ToString::to_string),
            "api_debug":format!("{:?}", api.as_ref().err()),
            "cli_exit":cli.status.code(), "difference":difference,
        }))
        .unwrap(),
    )
    .unwrap();
    let bytes = api.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {root:?}"));
    assert!(cli.status.success(), "CLI failed: {root:?}");
    assert_eq!(bytes, fs::read(cli_path).unwrap(), "API/CLI: {root:?}");
    crate::golden::compare_ordered_bytes_generator_only(&native, &bytes)
        .unwrap_or_else(|error| panic!("{label}: {error}; artifacts {root:?}"));
}

// GCode.cpp:1774-1778/1866: these ordered inputs have no first-layer extrusions.
// A successful diagnostic test is not a Native/Ares parity PASS.
#[tokio::test(flavor = "current_thread")]
async fn rejected_orders_diagnostic_only() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("mesh-edge Native unavailable");
        return;
    };
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
    for (label, input) in [
        (
            "odd-reverse",
            include_bytes!("../../../../../tests/parity/mesh-edge-pairing/odd-reverse.3mf")
                .as_slice(),
        ),
        (
            "odd-rotated",
            include_bytes!("../../../../../tests/parity/mesh-edge-pairing/odd-rotated.3mf")
                .as_slice(),
        ),
    ] {
        let root = tempfile::Builder::new()
            .prefix("mesh-edge-diagnostic-")
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
                "classification":"diagnostic-only", "parity_pass":false,
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
        assert_eq!(native_result["return_code"], slicing_error, "{root:?}");
        let captured = error.captured.unwrap();
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) {
                slicing_error.rem_euclid(256)
            } else {
                slicing_error
            }),
            "{root:?}"
        );
        assert_eq!(native_result["error_string"], slicing_message, "{root:?}");
        assert!(!output.join("plate_1.gcode").exists(), "{root:?}");
    }
}
