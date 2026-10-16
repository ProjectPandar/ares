//! Public RED for BrimEars per-object skirt admission.
//! Upstream: Print.cpp:2780-2977,2847-2971; Brim.cpp:354-412,537-551,590-634.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/brim-ears.3mf");
const INPUT_SHA256: &str = "ff8389789f3795235be0cc11666e5acb301e3f5797bbf915366a694495d31593";

#[test]
fn orca_parity_per_object_skirt_brim_ears_red() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let hash = sha(INPUT);
    assert_eq!(hash, INPUT_SHA256);
    let root = tempfile::Builder::new()
        .prefix("per-object-skirt-brim-ears-red-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    let native_dir = root.join("native");
    fs::write(&input, INPUT).unwrap();
    fs::create_dir(&native_dir).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_dir.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let text = String::from_utf8(native.clone()).unwrap();
    assert!(text.contains(";TYPE:Brim\n"), "Native BrimEars active");
    assert!(
        text.matches(";TYPE:Skirt\n").count() >= 2,
        "Native per-object skirt active"
    );
    fs::write(root.join("native.gcode"), &native).unwrap();

    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            INPUT,
            ares_core::GenerationMetadata::deterministic(2026, 10, 17, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|error| format!("API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&native, gcode));

    let cli_path = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&cli_path);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&native, gcode));
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": hash,
            "native_sha256": sha(&native),
            "native_brim_blocks": text.matches(";TYPE:Brim\n").count(),
            "native_skirt_blocks": text.matches(";TYPE:Skirt\n").count(),
            "api_error": api.as_ref().err().map(|error| format!("{error:?}")),
            "api_comparison": api_comparison,
            "cli_code": cli.status.code(),
            "cli_comparison": cli_comparison,
            "strict_parity_required": true
        }))
        .unwrap(),
    )
    .unwrap();

    api_comparison.unwrap_or_else(|error| panic!("API strict RED at {root:?}: {error}"));
    assert!(cli.status.success(), "CLI strict RED at {root:?}");
    cli_comparison.unwrap_or_else(|error| panic!("CLI strict RED at {root:?}: {error}"));
    assert_eq!(api.unwrap(), cli_bytes.unwrap(), "API/CLI raw equality");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
