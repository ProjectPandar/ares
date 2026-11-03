//! PrintApply.cpp:1032-1088 / PrintObject.cpp:2223-2251,3973-3995.
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
                crate::native_environment::skipped("modifier top-shell Native unavailable");
                return;
            };
            compare(
                &runner,
                stringify!($name),
                include_bytes!(concat!(
                    "../../../../../tests/parity/modifier-top-shell/",
                    $input,
                    ".3mf"
                )),
            )
            .await;
        }
    };
}

case!(classic_none, "classic-none");
case!(classic_zero, "classic-zero");
case!(classic_one, "classic-one");
case!(classic_four, "classic-four");
case!(classic_equal, "classic-equal");
case!(classic_outside, "classic-outside");
case!(classic_overlap, "classic-overlap");
case!(classic_horizontal_none, "classic-horizontal-none");
case!(classic_horizontal_one, "classic-horizontal-one");
case!(classic_horizontal_four, "classic-horizontal-four");
case!(arachne_none, "arachne-none");
case!(arachne_zero, "arachne-zero");
case!(arachne_one, "arachne-one");
case!(arachne_four, "arachne-four");
case!(arachne_equal, "arachne-equal");
case!(arachne_outside, "arachne-outside");
case!(arachne_overlap, "arachne-overlap");

async fn compare(runner: &OrcaRunner, label: &str, input: &[u8]) {
    let mut builder = tempfile::Builder::new();
    builder.prefix("modifier-top-shell-");
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
            label: format!("modifier-top-shell/{label}"),
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
            "label":label, "input_sha256":sha(input), "native_sha256":sha(&native),
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
