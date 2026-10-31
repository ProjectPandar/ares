//! FillBase.cpp:996-1241,1432-1565,1580-1594; EdgeGrid.hpp:172 onwards.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn support_touching_grid_s_c0_public_api_cli() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/support-touching-grid/S-C0.3mf");
    let bytes = fs::read(fixture).unwrap();
    assert_eq!(
        sha(&bytes),
        "9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1"
    );
    let root = tempfile::Builder::new()
        .prefix("support-touching-grid-s-c0-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "args":args, "input_sha256":sha(&bytes),
            "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "source":"FillBase.cpp:996-1241/1432-1565/1580-1594; EdgeGrid.hpp:172",
            "seeded":std::env::var_os("ARES_PARITY_SEEDED_REFERENCE").is_some()
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native completion {root:?}");
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&reference)
            .unwrap()
            .contains(";TYPE:Sparse infill\n")
    );

    // Capture CLI exit status and stderr before invoking the public API.
    let binary = env!("CARGO_BIN_EXE_ares");
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    fs::write(
        root.join("admission.json"),
        serde_json::to_vec_pretty(&json!({
            "native_return_code":result["return_code"],
            "native_gcode_sha256":sha(&reference),"cli_exit":cli.status.code(),
            "cli_binary_sha256":sha(&fs::read(binary).unwrap()),
            "api_call":"slice_project_plate_validated", "strict_parity_required":true
        }))
        .unwrap(),
    )
    .unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let api = api.unwrap_or_else(|error| panic!("public API admission {root:?}: {error:?}"));
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    assert!(cli.status.success(), "CLI admission {root:?}: {cli:?}");
    let cli_bytes = fs::read(output).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "api_sha256":sha(&api),"cli_sha256":sha(&cli_bytes),
            "api_equals_cli":api==cli_bytes
        }))
        .unwrap(),
    )
    .unwrap();
    api_comparison.unwrap_or_else(|error| panic!("API strict comparison {root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("CLI strict comparison {root:?}: {error}"));
    assert_eq!(api, cli_bytes, "public API/CLI raw equality {root:?}");
}
