//! Per-object skirt brim-type admission RED.
//! Upstream: Print.cpp::_make_skirt:2780-2977,2847-2971;
//! Brim.cpp:448-460,590-634,964-977.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn per_object_skirt_inner_only_brim_red() {
    run_case("inner-only", false);
}

#[test]
fn per_object_skirt_outer_only_control() {
    run_case("outer-only", true);
}

fn run_case(label: &str, control: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-skirt-brim-types-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = crate::runner::repo_root()
        .join("tests/parity/per-object-skirt-brim-types")
        .join(format!("{label}.3mf"));
    let project = fs::read(&input).unwrap();
    fs::write(root.join("input.3mf"), &project).unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_dir.to_str().unwrap().to_owned(),
            root.join("input.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let native_text = String::from_utf8(native.clone()).unwrap();
    assert_eq!(native_text.matches(";TYPE:Brim\n").count(), 2);
    assert_eq!(native_text.matches(";TYPE:Skirt\n").count(), 2);
    fs::write(root.join("native.gcode"), &native).unwrap();

    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &project,
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
        .arg(root.join("input.3mf"))
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
            "case": label,
            "control": control,
            "input_sha256": sha(&project),
            "native_sha256": sha(&native),
            "native_brim_blocks": native_text.matches(";TYPE:Brim\n").count(),
            "native_skirt_blocks": native_text.matches(";TYPE:Skirt\n").count(),
            "api_error": api.as_ref().err().map(|error| format!("{error:?}")),
            "api_comparison": api_comparison,
            "cli_code": cli.status.code(),
            "cli_comparison": cli_comparison,
            "strict_parity_required": true,
            "upstream": "Print.cpp:2780-2977,2847-2971; Brim.cpp:448-460,590-634,964-977"
        }))
        .unwrap(),
    )
    .unwrap();

    // Both cases assert the target equality; InnerOnly is the intentional RED.
    api_comparison.unwrap_or_else(|error| panic!("{label}: API strict RED at {root:?}: {error}"));
    assert!(cli.status.success(), "{label}: CLI strict RED at {root:?}");
    cli_comparison.unwrap_or_else(|error| panic!("{label}: CLI strict RED at {root:?}: {error}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "{label}: API/CLI raw equality"
    );
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
