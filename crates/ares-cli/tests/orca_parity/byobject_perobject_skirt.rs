//! Print.cpp:2548-2557,2780-2977; GCode.cpp:3280-3374,4465-4483,5334.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn byobject_perobject_skirt_target() {
    check("target");
}

#[test]
fn byobject_perobject_skirt_bylayer_control() {
    check("bylayer-control");
}

#[test]
fn byobject_perobject_skirt_combined_control() {
    check("combined-control");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/byobject-perobject-skirt");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let case = &manifest["cases"][label];
    let settings = crate::runner::application::settings(&bytes).unwrap();
    assert_eq!(settings["print_sequence"], case["sequence"]);
    assert_eq!(settings["skirt_type"], case["skirt"]);
    for (key, value) in [
        ("brim_type", "no_brim"),
        ("combine_brims", "0"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("enable_prime_tower", "0"),
        ("gcode_comments", "1"),
    ] {
        assert_eq!(settings[key], value);
    }
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 1);
    let root = tempfile::Builder::new()
        .prefix(&format!("byobject-perobject-skirt-{label}-"))
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
            "input_sha256":sha(&bytes), "args":args,
            "source":"Print.cpp:2548-2557,2780-2977; GCode.cpp:3280-3374,4465-4483,5334"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let native = std::str::from_utf8(&reference).unwrap();
    assert!(
        native
            .lines()
            .any(|line| line.starts_with("G1 X") && line.contains(" E"))
    );
    if label == "target" {
        assert!(native.contains("; total layer number: 40\n"));
        let skirts: Vec<_> = native
            .match_indices(";TYPE:Skirt\n")
            .map(|(i, _)| i)
            .collect();
        assert_eq!(skirts.len(), 2);
        let origin = native
            .find("; move to origin position for next object")
            .unwrap();
        let first = native
            .find("; printing object cube10.stl id:0 copy 0")
            .unwrap();
        // Sequential process_layers reuses local object id 0 for the next stack.
        let second = origin
            + native[origin..]
                .find("; printing object cube10.stl id:0 copy 0")
                .unwrap();
        assert!(skirts[0] < first && first < origin && origin < skirts[1] && skirts[1] < second);
        for (start, end) in [(skirts[0], first), (skirts[1], second)] {
            assert!(
                native[start..end]
                    .lines()
                    .any(|line| line.starts_with("G1 X") && line.contains(" E"))
            );
        }
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
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
            "api_error":api.as_ref().err().map(|error| format!("{error:?}")),
            "cli_code":cli.status.code(), "cli_output_exists":output.exists()
        }))
        .unwrap(),
    )
    .unwrap();
    let api =
        api.unwrap_or_else(|error| panic!("Native-valid input must slice: {error:?}; {root:?}"));
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    let cli_bytes = fs::read(&output).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "raw_api_cli_equal":api==cli_bytes,"cli_code":cli.status.code()
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI {root:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}
