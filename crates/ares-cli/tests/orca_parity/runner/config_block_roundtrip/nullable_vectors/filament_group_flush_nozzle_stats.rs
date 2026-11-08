//! PrintConfig.cpp:724–731,940–972 nozzle statistics at public slicing boundaries.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs};

fn fields(text: &str) -> BTreeMap<&str, &str> {
    text.lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| {
            matches!(
                *key,
                "filament_map" | "filament_nozzle_map" | "filament_volume_map"
            )
        })
        .collect()
}

fn fixture(name: &str, stats: &str, supported: bool) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let dir = repo_root().join("tests/parity/filament-group-flush-context");
    let manifest: Value =
        serde_json::from_slice(&fs::read(dir.join("provenance.json")).unwrap()).unwrap();
    let original = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "x2d_pla_three_objects_default_purge")
        .unwrap();
    let source = fs::read(dir.join(original["input"].as_str().unwrap())).unwrap();
    assert_eq!(sha(&source), original["sha256"].as_str().unwrap());
    let input = super::input::settings_variant(
        &source,
        json!({"extruder_nozzle_stats": [stats, "Standard#1"]})
            .as_object()
            .unwrap(),
    );
    let input_sha = sha(&input);
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-nozzle-stats",
            &[
                "PrintConfig.hpp:618-630 stable NozzleVolumeType values",
                "PrintConfig.cpp:724-731,940-972 registered names and std::atoi counts",
                "ToolOrdering.cpp:1508-1530 bounded static nozzle inventory",
            ],
        ),
        name,
        &input,
        &input_sha,
        false,
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &input,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let api_error = api.as_ref().err().map(ToString::to_string);
    let cli_error = String::from_utf8_lossy(&cli.stderr);
    let native_fields = fields(&reference.native);
    assert_eq!(native_fields.len(), 3);
    fs::write(
        reference.root.join("nozzle-stats-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":name, "input_sha256":input_sha, "source_sha256":sha(&source),
            "requested_stats":stats, "supported":supported,
            "native_fields":native_fields, "native_sha256":sha(reference.native.as_bytes()),
            "native_exported_stats":reference.native.lines().find_map(|line| line.strip_prefix("; extruder_nozzle_stats = ")),
            "api_error":api_error, "cli_status":cli.status.code(), "cli_stderr":cli_error,
            "scope":"public support boundary and current Native maps for supported inputs; no new solver, volume, or inventory scoring credit",
            "whole_output_parity_claimed":false
        }))
        .unwrap(),
    )
    .unwrap();
    if !supported {
        assert!(
            api_error.is_some_and(
                |error| error.contains("unsupported automatic filament grouping context")
            ),
            "valid unsupported nozzle type was not rejected; {:?}",
            reference.root
        );
        assert!(!cli.status.success(), "CLI; {:?}", reference.root);
        assert!(cli_error.contains("unsupported automatic filament grouping context"));
        return;
    }
    let api = api.unwrap();
    assert!(cli.status.success(), "{cli_error}; {:?}", reference.root);
    let cli = fs::read(cli_path).unwrap();
    let whole_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let whole_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    fs::write(
        reference
            .root
            .join("supported-nozzle-stats-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":name, "input_sha256":input_sha,
            "api_sha256":sha(&api), "cli_sha256":sha(&cli),
            "public_api_cli_complete_bytes_equal":api == cli,
            "strict_whole_api_comparison":whole_api, "strict_whole_cli_comparison":whole_cli,
            "native_fields":native_fields,
            "api_fields":fields(std::str::from_utf8(&api).unwrap()),
            "cli_fields":fields(std::str::from_utf8(&cli).unwrap()),
            "whole_output_parity_claimed":false
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api, cli, "complete API/CLI bytes; {:?}", reference.root);
    assert_eq!(fields(std::str::from_utf8(&api).unwrap()), native_fields);
    assert_eq!(fields(std::str::from_utf8(&cli).unwrap()), native_fields);
}

#[test]
fn x2d_nozzle_stats_accept_decimal_prefix_counts() {
    for (name, wire) in [
        ("count-suffix", "Standard#1x"),
        ("signed-count-prefix", "Standard# \t+1tail"),
    ] {
        fixture(name, wire, true);
    }
}

#[test]
fn x2d_nozzle_stats_skip_malformed_and_unknown_tokens() {
    for (name, wire) in [("missing-count", "Standard"), ("unknown-type", "Unknown#1")] {
        fixture(name, wire, true);
    }
}

#[test]
fn x2d_valid_other_nozzle_stats_are_unsupported() {
    for (id, kind) in [
        ("tpu-high-flow", "TPU High Flow"),
        ("e3d-high-flow", "E3D High Flow"),
        ("extra-high-flow", "Extra High Flow"),
    ] {
        fixture(id, &format!("{kind}#1"), false);
    }
}
