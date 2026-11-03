//! `GCode.cpp:2901-2904`: by-layer usage follows emitted tools, not model defaults.

use super::{ExportedCase, OrcaRunner, cache_identity};
use std::fs;

fn check(name: &str, input: &[u8], sha256: &str, expected_usage: [bool; 2]) {
    assert_eq!(cache_identity::raw_digest(input), sha256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-used-extruder-follows-emitted-tools-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&input_path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: format!("j1-used-extruder-follows-emitted-tools/{name}"),
            project: input.to_vec(),
            project_path: input_path,
            output_dir: output,
        })
        .unwrap();
    let (actual, comparison) = if name == "role2" {
        let outcome = crate::compare_case(&sliced);
        let artifacts = outcome.artifacts.as_ref().unwrap();
        (
            fs::read(artifacts.join("ares.gcode")).unwrap(),
            Some((outcome.status, outcome.detail, artifacts.clone())),
        )
    } else {
        let actual = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(crate::runner::application::slice_project(
                input,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                ares_core::SlicerContext::new(0),
            ))
            .unwrap();
        fs::write(directory.join("ares.gcode"), &actual).unwrap();
        fs::write(directory.join("orca.gcode"), &sliced.reference).unwrap();
        (actual, None)
    };
    let native_witness = witness(&sliced.reference);
    let ares_witness = witness(&actual);
    fs::write(
        directory.join("usage-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": sha256,
            "native_sha256": cache_identity::raw_digest(&sliced.reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native": native_witness,
            "ares": ares_witness,
            "complete_comparison": comparison,
            "assertion_scope": if name == "role2" { "complete bytes; generated-by identity only; M73/footer retained" } else { "exact usage, initial and tool/flush witness only; known unrelated complete-byte residual retained separately" }
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_witness["usage"], serde_json::json!(expected_usage));
    let expected_initial = usize::from(name == "role2" || name == "both-used");
    assert_eq!(native_witness["initial_extruder"], expected_initial);
    assert_eq!(ares_witness["initial_extruder"], expected_initial);
    let expected_commands = match name {
        "role2" => vec!["T1", "T1", "G1 X324 E8.5437 F6840.0", "T1"],
        "both-used" => vec![
            "T1",
            "T0",
            "G1 X0 E8.5437 F6840.0",
            "T1",
            "G1 X324 E8.5437 F6840.0",
            "T1",
            "T0",
            "T1",
            "T0",
        ],
        "role1" | "by-object" => vec!["T0", "T0", "G1 X0 E8.5437 F6840.0", "T0"],
        _ => unreachable!(),
    };
    assert_eq!(
        native_witness["tool_and_flush_commands"],
        serde_json::json!(expected_commands)
    );
    if let Some((status, detail, _)) = comparison {
        assert_eq!(status, "PASS", "{detail}; {directory:?}");
    }
    assert_eq!(ares_witness, native_witness);
}

fn witness(bytes: &[u8]) -> serde_json::Value {
    let text = std::str::from_utf8(bytes).unwrap();
    let usage = [0, 1].map(|tool| {
        let prefix = format!("; --- T{tool}: ");
        text.lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap()
            .parse::<bool>()
            .unwrap()
    });
    let initial = text
        .lines()
        .find_map(|line| line.strip_prefix("; --- initial_extruder: "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let commands = text
        .lines()
        .filter(|line| {
            matches!(*line, "T0" | "T1")
                || line.starts_with("G1 X324 E8.5437")
                || line.starts_with("G1 X0 E8.5437")
        })
        .collect::<Vec<_>>();
    serde_json::json!({"usage": usage, "initial_extruder": initial, "tool_and_flush_commands": commands})
}

#[test]
fn j1_by_layer_role2_usage_matches_complete_output() {
    check(
        "role2",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role2.3mf"
        ),
        "7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4",
        [false, true],
    );
}

// Coordinator-approved witness-only control: frozen whole-file first difference
// byte2449/line165 is Native M73 P41 versus Ares WIPE_START, unrelated to mask.
#[test]
fn j1_by_layer_role1_usage_witness() {
    check(
        "role1",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role1.3mf"
        ),
        "e74bf5ed81edcf4c00ea36be06967e1447ee4d61d25cd20b23fd39f16f6227e5",
        [true, false],
    );
}

// Frozen whole-file first difference byte2384/line189: Native FORCE_RESUME_FAN_SPEED
// versus Ares blank/change T0 -> T1. Only this control's exact witness is asserted.
#[test]
fn j1_by_layer_both_used_witness() {
    check(
        "both-used",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
        "12cf6020f118d5a2c5b5aa081f5dd9a4a3c043f0c3788558c13fa80687a4f3bd",
        [true, true],
    );
}

// Deferred by-object control: same unrelated byte2449/line165 residual as role1.
// Preserve exact usage/initial/tool witness; do not claim complete-byte parity.
#[test]
fn j1_by_object_usage_witness_unchanged() {
    check(
        "by-object",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/by-object.3mf"
        ),
        "828e4056bfdeaaed2f028fdbd596355a2f2fa56d0c8fab060f89268435823633",
        [true, false],
    );
}
