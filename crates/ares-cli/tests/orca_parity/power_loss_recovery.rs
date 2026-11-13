//! Power-loss-recovery emission (`GCode.cpp:4777-4779`,
//! `:3359-3361`, `:3440-3441`; emitter `GCodeWriter.cpp:505-522`):
//! with `enable_power_loss_recovery = enable` Native writes the enable
//! command at the first non-first layer and the disable command after
//! the layer loop (ByLayer) or per finished object (ByObject — two
//! independent objects with `print_sequence = by object` and
//! `gcode_flavor = marlin2` pin `GCode.cpp:3355-3367`: each object's
//! second layer re-enables and each completion disables), spelled
//! `M1003 S1`/`M1003 S0` on BBL printers and `M413 S1`/`M413 S0` for
//! `gcode_flavor = marlin2`, other flavors emitting nothing, and the
//! verbatim ` ; set Power-loss Recovery` suffix under
//! `gcode_comments = 1`. Ares parses the enum but never emits, so the
//! four target legs must FAIL on the strict whole-output compare (the
//! first difference is the layer-2 enable line); `empty-klipper` and
//! the default-mode control must PASS.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

#[path = "power_loss_recovery/planner_boundaries.rs"]
mod planner_boundaries;
#[path = "power_loss_recovery/queue_flush.rs"]
mod queue_flush;

const BY_LAYER: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/target-by-layer.3mf");
const BY_LAYER_SHA256: &str = "92f48044969ff8c862077d0cedeb29b930c71c9b64f66ac17ff28659cf7cae4b";
const BY_OBJECT: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/target-by-object.3mf");
const BY_OBJECT_SHA256: &str = "f77c65ab681ceb4bdce7be3f1bd803cf08779b05f9c924e58e2ace7fa7a98fa0";
const MARLIN2: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/target-marlin2.3mf");
const MARLIN2_SHA256: &str = "713ce074266ef0a26682e7d3ff74db2e138f487555afa8cd2d81268272100354";
const COMMENTS: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/target-comments.3mf");
const COMMENTS_SHA256: &str = "0b47be231f6e6d3f00f36a759a58f277351661a7e7ba996ae8ef940f2706a0e7";
const EMPTY_KLIPPER: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/empty-klipper.3mf");
const EMPTY_KLIPPER_SHA256: &str =
    "9e5e70bf2cb38688d41c2eec5095e486c62f43d0bd578e254fce632271428d53";
const CONTROL: &[u8] = include_bytes!("../../../../tests/parity/power-loss-recovery/control.3mf");
const CONTROL_SHA256: &str = "19fb31b00554291f5d94378152e9aa5eb085c42960c136f3aa77a2b372d4ffe4";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn slice_native(runner: &OrcaRunner, root: &std::path::Path, input: &[u8], prefix: &str) -> String {
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    fs::read_to_string(output.join("plate_1.gcode")).unwrap()
}

fn compare(
    root: &std::path::Path,
    label: &str,
    input: &[u8],
    reference: &str,
    source: &str,
) -> crate::ParityOutcome {
    crate::artifacts::compare(
        root,
        label,
        Ok(input),
        Ok(reference.as_bytes()),
        json!({
            "kind":"power_loss_recovery",
            "input_sha256":sha256(input),
            "source":source,
        }),
    )
    .unwrap()
}

#[test]
fn power_loss_recovery_by_layer_matches_native() {
    assert_eq!(sha256(BY_LAYER), BY_LAYER_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, BY_LAYER, "plr-by-layer-");
    assert_eq!(reference.lines().filter(|l| *l == "M1003 S1").count(), 1);
    assert_eq!(reference.lines().filter(|l| *l == "M1003 S0").count(), 1);
    let outcome = compare(
        &root,
        "plr-by-layer",
        BY_LAYER,
        &reference,
        "GCode.cpp:4777-4779,3440-3441;GCodeWriter.cpp:505-522",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn power_loss_recovery_by_object_matches_native() {
    assert_eq!(sha256(BY_OBJECT), BY_OBJECT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, BY_OBJECT, "plr-by-object-");
    assert_eq!(reference.lines().filter(|l| *l == "M413 S1").count(), 2);
    assert_eq!(reference.lines().filter(|l| *l == "M413 S0").count(), 2);
    let outcome = compare(
        &root,
        "plr-by-object",
        BY_OBJECT,
        &reference,
        "GCode.cpp:4777-4779,3359-3361;GCodeWriter.cpp:505-522",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn power_loss_recovery_marlin2_matches_native() {
    assert_eq!(sha256(MARLIN2), MARLIN2_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, MARLIN2, "plr-marlin2-");
    assert_eq!(reference.lines().filter(|l| *l == "M413 S1").count(), 1);
    assert_eq!(reference.lines().filter(|l| *l == "M413 S0").count(), 1);
    let outcome = compare(
        &root,
        "plr-marlin2",
        MARLIN2,
        &reference,
        "GCode.cpp:4777-4779,3440-3441;GCodeWriter.cpp:513-518",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn power_loss_recovery_comments_matches_native() {
    assert_eq!(sha256(COMMENTS), COMMENTS_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, COMMENTS, "plr-comments-");
    assert_eq!(
        reference
            .lines()
            .filter(|l| *l == "M1003 S1 ; set Power-loss Recovery")
            .count(),
        1
    );
    assert_eq!(
        reference
            .lines()
            .filter(|l| *l == "M1003 S0 ; set Power-loss Recovery")
            .count(),
        1
    );
    let outcome = compare(
        &root,
        "plr-comments",
        COMMENTS,
        &reference,
        "GCodeWriter.cpp:519-520 (gcode_comments suffix)",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn power_loss_recovery_empty_klipper_matches_native() {
    assert_eq!(sha256(EMPTY_KLIPPER), EMPTY_KLIPPER_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, EMPTY_KLIPPER, "plr-klipper-");
    assert_eq!(reference.lines().filter(|l| l.contains("M413")).count(), 0);
    assert_eq!(reference.lines().filter(|l| l.contains("M1003")).count(), 0);
    let outcome = compare(
        &root,
        "plr-klipper",
        EMPTY_KLIPPER,
        &reference,
        "GCodeWriter.cpp:505-522 (non-BBL non-marlin2 emits nothing)",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn power_loss_recovery_control_matches_native() {
    assert_eq!(sha256(CONTROL), CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, CONTROL, "plr-control-");
    assert_eq!(reference.lines().filter(|l| l.contains("M1003")).count(), 0);
    let outcome = compare(
        &root,
        "plr-control",
        CONTROL,
        &reference,
        "GCodeWriter.cpp:508-509 (printer_configuration emits nothing)",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
