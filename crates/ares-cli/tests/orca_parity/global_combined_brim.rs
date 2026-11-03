//! Print.cpp:2532-2543; Brim.cpp:958-1018; GCode.cpp:4486-4531.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn global_combined_brim_target() {
    check("target");
}

#[test]
fn global_combined_brim_control() {
    check("control");
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
    let fixture = crate::runner::repo_root().join("tests/parity/global-combined-brim");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("print_sequence", "by layer"),
        ("skirt_type", "combined"),
        ("brim_type", "outer_only"),
        ("combine_brims", if label == "target" { "1" } else { "0" }),
        ("brim_width", "8"),
        ("brim_object_gap", "0"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("enable_prime_tower", "0"),
    ] {
        assert_eq!(settings[key], value);
    }
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 1);
    let root = tempfile::Builder::new()
        .prefix(&format!("global-combined-brim-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native-first")).unwrap();
    fs::create_dir(root.join("native-second")).unwrap();
    let slice_args = |outputdir: &str| {
        vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            root.join(outputdir).to_str().unwrap().into(),
            root.join("input.3mf").to_str().unwrap().into(),
        ]
    };
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "input_sha256":sha(&bytes), "args":slice_args("native-first"),
            "stability_double_run":"native-second",
            "source":"Print.cpp:3849-3867; Brim.cpp:941-1010; GCode.cpp:5991-6082"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(
        runner.bin(),
        slice_args("native-first"),
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    stages::run(
        runner.bin(),
        slice_args("native-second"),
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    // Stability premise: the Native producer must be deterministic on
    // this input under the generator-line normalization the acceptance
    // comparison uses.
    let normalize = |raw: &[u8]| {
        golden::normalize_one_generator_line(raw, golden::GeneratorKind::Orca)
            .unwrap_or_else(|error| panic!("Native generator line: {error}"))
    };
    let first = fs::read(root.join("native-first/plate_1.gcode")).unwrap();
    let second = fs::read(root.join("native-second/plate_1.gcode")).unwrap();
    assert_eq!(
        normalize(&first),
        normalize(&second),
        "Native slice is not deterministic"
    );
    let reference = first;
    let native = std::str::from_utf8(&reference).unwrap();
    assert!(
        native
            .lines()
            .any(|line| line.starts_with("G1 X") && line.contains(" E"))
    );
    // No frozen Native brim-block count premise: the beta oracle emits 2
    // blocks on the target where this froze 1; the count is compared only
    // as part of the whole-output comparison below, never waived on its
    // own.
    assert_eq!(native.matches(";TYPE:Skirt\n").count(), 1);
    for key in [
        "brim_type = outer_only",
        "skirt_type = combined",
        "print_sequence = by layer",
    ] {
        assert!(native.contains(&format!("; {key}\n")));
    }
    assert!(native.contains(&format!(
        "; combine_brims = {}\n",
        if label == "target" { 1 } else { 0 }
    )));
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let output = root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
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
    // Per-instance group-brim premise: ares must emit exactly as many
    // ;TYPE:Brim blocks as the Native reference produced in this run
    // (runtime cross-producer equality, not a frozen count).
    assert_eq!(
        std::str::from_utf8(&api)
            .unwrap()
            .matches(";TYPE:Brim\n")
            .count(),
        native.matches(";TYPE:Brim\n").count(),
        "ares per-instance brim grouping {root:?}"
    );
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
