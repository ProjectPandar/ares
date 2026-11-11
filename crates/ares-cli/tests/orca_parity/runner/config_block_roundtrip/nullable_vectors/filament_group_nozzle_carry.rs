//! `ToolOrderUtils.cpp:1463-1488`: retain the last nozzle for every visited head.

use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::GenerationMetadata;
use serde_json::json;
use std::fs;

#[test]
fn h2c_manual_nozzle_carry_nonzero_change_time_matches_native() {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let directory = repo_root().join("tests/parity/h2c-nozzle-carryover");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("provenance.json")).unwrap()).unwrap();
    let project = fs::read(directory.join(provenance["input"].as_str().unwrap())).unwrap();
    let input_sha = sha(&project);
    assert_eq!(input_sha, provenance["input_sha256"].as_str().unwrap());
    let (native, _) = native_stream_case::qualify(
        (
            "filament-group-nozzle-carry",
            &["ToolOrderUtils.cpp:1463-1488 grouping evaluation per-head nozzle carry"],
        ),
        "h2c-nonzero-change-time",
        &project,
        &input_sha,
        false,
    );
    let usage = native
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let used = usage
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap() > 0.0)
        .collect::<Vec<_>>();
    fs::write(
        native.root.join("native-carryover-premises.json"),
        serde_json::to_vec_pretty(&json!({
            "provenance": provenance, "input_sha256": input_sha,
            "native_usage_mm": usage, "native_used_filaments": used,
            "strict_full_output_required": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(used, [true; 3], "Native usage: {:?}", native.root);
    for (key, expected) in [
        ("extruder_max_nozzle_count", "1,6"),
        ("filament_map", "1,2,2"),
        ("filament_nozzle_map", "0,2,1"),
        ("filament_volume_map", "0,0,0"),
        ("machine_load_filament_time", "30"),
        ("machine_unload_filament_time", "30"),
    ] {
        assert!(
            native.native.contains(&format!("; {key} = {expected}\n")),
            "Native {key}: {:?}",
            native.root
        );
    }
    native.compare(GenerationMetadata::deterministic(2026, 11, 10, 2, 0, 0));
}
