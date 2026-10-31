//! Per-object skirts with combined brims and multi-instance groups
//! (`Print.cpp:2646-2988:: _make_skirt` per-object path — occupied
//! points include every instance shift `:2797-2803`; `make_group_brims`
//! contact-combining `:2886-2957`; `GCode.cpp:4458-4483,4486-4540`
//! per-object skirt-group and brim emission `:5334-5335`).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn per_object_skirt_combine_multi_instance_far() {
    check("multi-instance-far", 2, 2);
}

#[test]
fn per_object_skirt_combine_multi_instance_contact() {
    check("multi-instance-contact", 1, 1);
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str, expected_groups: usize, expected_brims: usize) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/per-object-skirt-combine");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("skirt_type", "perobject"),
        ("skirt_loops", "2"),
        ("skirt_height", "1"),
        ("skirt_distance", "6"),
        ("min_skirt_length", "0"),
        ("draft_shield", "disabled"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("brim_type", "outer_only"),
        ("brim_width", "8"),
        ("combine_brims", "1"),
        ("brim_object_gap", "0"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
    ] {
        assert_eq!(settings[key], value);
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-skirt-combine-{label}-"))
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
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "args":args,"input_sha256":sha(&bytes),
            "source":"Print.cpp:2646-2988; GCode.cpp:4458-4540,5334-5335"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(text.contains("; skirt_type = perobject\n"));
    assert!(text.contains("; combine_brims = 1\n"));
    let skirt_blocks = text.split(";TYPE:Skirt\n").skip(1).count();
    assert_eq!(
        skirt_blocks, expected_groups,
        "Native skirt groups {root:?}"
    );
    let brim_blocks = text.split(";TYPE:Brim\n").skip(1).count();
    assert_eq!(brim_blocks, expected_brims, "Native brim blocks {root:?}");
    fs::write(
        root.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"input_sha256":sha(&bytes),"native_sha256":sha(&reference),
            "skirt_groups":skirt_blocks,"brim_blocks":brim_blocks
        }))
        .unwrap(),
    )
    .unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 10, 29, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
            "api_comparison":api_comparison,"cli_code":cli.status.code(),
            "cli_comparison":cli_comparison,"cli_binary_sha256":sha(&fs::read(binary).unwrap()),
            "strict_parity_required":true
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(api.is_ok(), "public API admission RED {root:?}: {api:?}");
    assert!(cli.status.success(), "CLI admission RED {root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    cli_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "API/CLI raw equality {root:?}"
    );
}
