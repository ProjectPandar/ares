//! Fill.cpp:1440-1447 -> FillBase.cpp:311-313 -> FillHoneycomb.hpp:53.
//! Public failures: altered input/flags, absent bridge activity, unstable Native,
//! ordinary Sparse drift, API/CLI drift, or strict complete-output drift.
//! Progress/M73 remains in the unchanged whole-file acceptance comparator.

use crate::runner::{ExportedCase, OrcaRunner};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command};

#[path = "honeycomb_anchoring_zaa/observe.rs"]
mod observe;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    sha: &'static str,
    top: usize,
    enabled: bool,
    flag: bool,
}

const TARGET: Case = Case {
    name: "top6-enabled-flag1",
    sha: "078df2b3f6ffa92167b8c044408b02beaf5534a7bcebdbe6bebc606b074a9812",
    top: 6,
    enabled: true,
    flag: true,
};
const FLAG_FALSE: Case = Case {
    name: "top6-enabled-flag0",
    sha: "c1a68d0452738de551592b6bebdce8db631379320c3d25ea6f74c4c4a54f66f5",
    top: 6,
    enabled: true,
    flag: false,
};
const DISABLED_ZAA: Case = Case {
    name: "top6-disabled-flag1",
    sha: "662a314cc90f2a83c3a79de9e4c742b7bc29cd6eea30975bded1e1037079fbd7",
    top: 6,
    enabled: false,
    flag: true,
};
const PHASE_ZERO: Case = Case {
    name: "top7-enabled-flag1",
    sha: "6604daa38d5c7dd73f39311c7094e988e0b87317968ceab9d817650497d669df",
    top: 7,
    enabled: true,
    flag: true,
};
const PHASE_TWO: Case = Case {
    name: "top8-enabled-flag1",
    sha: "a838438b5bd0b7c5895ef5b167f77bee0cce1519a7babe3425bc6ad587e12753",
    top: 8,
    enabled: true,
    flag: true,
};

#[test]
fn top6_regional_direction_matches_native_complete_output() {
    verify(TARGET);
}

#[test]
fn flag_false_preserves_anchoring_and_complete_output() {
    verify(FLAG_FALSE);
}

#[test]
fn disabled_zaa_preserves_anchoring_and_complete_output() {
    verify(DISABLED_ZAA);
}

#[test]
fn phase_zero_preserves_anchoring_and_complete_output() {
    verify(PHASE_ZERO);
}

#[test]
fn phase_two_preserves_anchoring_and_complete_output() {
    verify(PHASE_TWO);
}

struct Recorded {
    root: PathBuf,
    native_sha: String,
    api_sha: String,
    status: &'static str,
    detail: String,
    cli_difference: Option<String>,
}

fn frozen(case: Case) -> Vec<u8> {
    let project = fs::read(
        crate::runner::repo_root()
            .join("tests/parity/honeycomb-anchoring-zaa")
            .join(format!("{}.3mf", case.name)),
    )
    .unwrap();
    assert_eq!(
        observe::hash(&project),
        case.sha,
        "frozen input {}",
        case.name
    );
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].instances().len(), 1);
    let region = &loaded.settings().process.region;
    assert_eq!(region.zaa_enabled.0, case.enabled);
    assert_eq!(region.zaa_dont_alternate_fill_direction.0, case.flag);
    assert_eq!(
        region.sparse_infill_pattern,
        ares_core::ProcessInfillPattern::Honeycomb
    );
    assert_eq!(region.fill_multiline.0, 2);
    assert_eq!(
        usize::try_from(region.top_shell_layers.0).unwrap(),
        case.top
    );
    project
}

fn verify(case: Case) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = frozen(case);
    let runs: Vec<_> = (1..=2)
        .map(|repeat| run_once(&mut runner, case, repeat, &project))
        .collect();
    assert_eq!(
        runs[0].native_sha, runs[1].native_sha,
        "fresh Native repeat complete bytes: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].api_sha, runs[1].api_sha,
        "public API repeat raw bytes: {:?}",
        runs[0].root
    );
    for run in &runs {
        assert_eq!(run.status, "PASS", "{}; {:?}", run.detail, run.root);
        assert!(
            run.cli_difference.is_none(),
            "CLI strict complete output: {:?}; {:?}",
            run.cli_difference,
            run.root
        );
    }
}

fn run_once(runner: &mut OrcaRunner, case: Case, repeat: usize, project: &[u8]) -> Recorded {
    runner.cache = tempfile::Builder::new()
        .prefix("honeycomb-anchoring-zaa-cache-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let native_root = tempfile::Builder::new()
        .prefix("honeycomb-anchoring-zaa-input-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let project_path = native_root.join("input.3mf");
    fs::write(&project_path, project).unwrap();
    let output_dir = native_root.join("native");
    fs::create_dir(&output_dir).unwrap();
    let sliced = runner
        .rerun_reference(&ExportedCase {
            label: format!("honeycomb-anchoring-zaa/{}/{repeat}", case.name),
            project: project.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&sliced);
    let root = outcome.artifacts.unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .arg("slice")
        .arg("-o")
        .arg(root.join("cli.gcode"))
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    let api_bytes = fs::read(root.join("ares.gcode")).unwrap();
    let cli_bytes = fs::read(root.join("cli.gcode")).unwrap();
    let native = &sliced.reference;
    let normalized =
        crate::golden::normalize_one_generator_line(native, crate::golden::GeneratorKind::Orca)
            .unwrap();
    let native_sha = observe::hash(&normalized);
    let api_sha = observe::hash(&api_bytes);
    let cli_difference =
        crate::golden::compare_ordered_bytes_generator_only(native, &cli_bytes).err();
    let native_sparse = observe::moves(native, "Sparse infill");
    let api_sparse = observe::moves(&api_bytes, "Sparse infill");
    let native_bridge = observe::moves(native, "Internal Bridge");
    let api_bridge = observe::moves(&api_bytes, "Internal Bridge");
    let native_layers = observe::positive_layers(&native_bridge);
    let api_layers = observe::positive_layers(&api_bridge);
    fs::write(
        root.join("anchoring-zaa-witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case.name, "repeat": repeat, "input_sha256": observe::hash(project),
            "enabled": case.enabled, "flag": case.flag, "top_shell_layers": case.top,
            "native_normalized_sha256": native_sha, "api_sha256": api_sha,
            "cli_sha256": observe::hash(&cli_bytes), "cli_exit": cli.status.code(),
            "native_bridge_layers": native_layers, "api_bridge_layers": api_layers,
            "native_bridge": native_bridge, "api_bridge": api_bridge,
            "native_sparse_rows": native_sparse.len(), "api_sparse_rows": api_sparse.len(),
            "sparse_equal": native_sparse == api_sparse, "api_cli_equal": api_bytes == cli_bytes,
            "fresh_native_cache": runner.cache, "status": outcome.status,
            "detail": outcome.detail, "cli_difference": cli_difference
        }))
        .unwrap(),
    )
    .unwrap();
    let reference = std::str::from_utf8(native).unwrap();
    for line in [
        "; CONFIG_BLOCK_END".to_owned(),
        "; sparse_infill_pattern = honeycomb".to_owned(),
        "; sparse_infill_density = 20%".to_owned(),
        "; infill_direction = 45".to_owned(),
        "; fill_multiline = 2".to_owned(),
        format!("; top_shell_layers = {}", case.top),
        format!("; zaa_enabled = {}", u8::from(case.enabled)),
        format!(
            "; zaa_dont_alternate_fill_direction = {}",
            u8::from(case.flag)
        ),
    ] {
        assert!(
            reference.lines().any(|found| found == line),
            "Native effective footer: {line}; {root:?}"
        );
    }
    // 10mm / 0.2mm = 50 object layers; top shells begin at physical 51 - count.
    let bridge_layers = std::collections::BTreeSet::from([51 - case.top]);
    assert_eq!(
        native_layers, bridge_layers,
        "Native bridge active: {root:?}"
    );
    assert_eq!(api_layers, bridge_layers, "API bridge active: {root:?}");
    assert!(!native_sparse.is_empty(), "Native Sparse active: {root:?}");
    assert_eq!(
        native_sparse, api_sparse,
        "ordinary Sparse unchanged: {root:?}"
    );
    assert!(cli.status.success(), "CLI execution: {root:?}");
    assert_eq!(api_bytes, cli_bytes, "public API/CLI raw bytes: {root:?}");
    Recorded {
        root,
        native_sha,
        api_sha,
        status: outcome.status,
        detail: outcome.detail,
        cli_difference,
    }
}
