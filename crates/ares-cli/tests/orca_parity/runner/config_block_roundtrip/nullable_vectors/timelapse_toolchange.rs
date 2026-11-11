//! `GCode.cpp:7476-7478,7513-7539`: non-wipe-tower Case B and disabled gate.

use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::GenerationMetadata;
use serde_json::json;
use std::fs;

fn compare(enabled: bool) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let directory = repo_root().join("tests/parity/no-tower-timelapse-gate");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("provenance.json")).unwrap()).unwrap();
    let case = provenance["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["farthest_point_timelapse"] == enabled)
        .unwrap();
    let name = case["id"].as_str().unwrap();
    let project = fs::read(directory.join(case["input"].as_str().unwrap())).unwrap();
    let input_sha = sha(&project);
    assert_eq!(input_sha, case["sha256"].as_str().unwrap());
    let settings = super::super::super::application::settings(&project).unwrap();
    assert_eq!(settings["printer_model"], "Bambu Lab X2D");
    assert_eq!(settings["enable_prime_tower"], "0");
    assert_eq!(settings["timelapse_type"], "0");
    assert_eq!(settings["filament_map"], json!(["1", "2"]));
    assert_eq!(
        settings["farthest_point_timelapse"],
        if enabled { "1" } else { "0" }
    );
    let (reference, _) = native_stream_case::qualify(
        (
            "no-tower-timelapse-gate",
            &["GCode.cpp:7476-7478,7513-7539 complete pre-toolchange gate"],
        ),
        name,
        &project,
        &input_sha,
        false,
    );
    let marker = reference
        .native
        .lines()
        .find(|line| line.starts_with("; TIMELAPSE_HEAD current="))
        .unwrap();
    let expected_marker = if enabled {
        "; TIMELAPSE_HEAD current=0 photo=1"
    } else {
        "; TIMELAPSE_HEAD current=1 photo=1"
    };
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let used = usage
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap() > 0.0)
        .collect::<Vec<_>>();
    fs::write(
        reference.root.join("native-gate-premises.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case, "input_sha256": input_sha,
            "first_native_marker": marker, "expected_marker": expected_marker,
            "native_used_filaments": used, "native_usage_mm": usage,
            "branch": if enabled { "Case B: current head differs from photo head" } else { "disabled: current head equals photo head" },
            "strict_full_output_required": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        used,
        [true, true],
        "Native dual-color usage: {:?}",
        reference.root
    );
    assert_eq!(
        marker, expected_marker,
        "Native gate premise: {:?}",
        reference.root
    );
    assert!(reference.native.contains("; filament_map = 1,2\n"));
    reference.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
}

#[test]
fn no_tower_case_b_inverts_current_photo_head() {
    compare(true);
}

#[test]
fn no_tower_disabled_uses_current_photo_head() {
    compare(false);
}
