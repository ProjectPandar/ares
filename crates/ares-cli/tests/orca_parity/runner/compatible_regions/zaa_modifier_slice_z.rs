//! Print.hpp:423-424 / PrintObjectSlice.cpp:24-47: all printing regions set slice Z.
//! Public failures: omitted modifier activation/minZ, inactive oracle, unstable
//! witness, changed ordinary control, API/CLI disagreement and whole-output drift.
use super::fixture::{self, Scope};
use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner},
};
use serde_json::json;
use std::{fs, path::PathBuf};

#[path = "zaa_modifier_slice_z_observe.rs"]
mod activity;
#[path = "zaa_modifier_slice_z_fixture.rs"]
mod input;
use input::Case;

#[test]
fn disabled_modifier_preserves_complete_output() {
    verify(Case::Disabled, None);
}

#[test]
fn inherited_equal_modifier_preserves_complete_output() {
    verify(Case::Equal, None);
}

#[test]
fn modifier_only_activation_preserves_complete_output() {
    verify(Case::ModifierOnly, Some(Case::Disabled));
}

#[test]
fn smaller_modifier_min_z_preserves_complete_output() {
    verify(Case::Smaller, Some(Case::Equal));
}

struct Recorded {
    root: PathBuf,
    status: &'static str,
    detail: String,
    native_sha: String,
    ares_sha: String,
    eligible_sha: String,
    cli_difference: Option<String>,
}

fn verify(case: Case, control: Option<Case>) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    fresh_cache(&mut runner);
    let (exported, fixture_proof) = fixture::prepare(&runner, Scope::Object, "rectilinear", false);
    let controls =
        control.map(|control| repeated(&mut runner, &exported.project, control, &fixture_proof));
    if let Some(controls) = &controls {
        for run in controls {
            assert_complete(run);
        }
    }
    let runs = repeated(&mut runner, &exported.project, case, &fixture_proof);
    if let Some(controls) = controls {
        assert_ne!(
            runs[0].eligible_sha, controls[0].eligible_sha,
            "Native modifier activity: {:?}",
            runs[0].root
        );
    }
    for run in &runs {
        assert_complete(run);
    }
}

fn fresh_cache(runner: &mut OrcaRunner) {
    runner.cache = tempfile::Builder::new()
        .prefix("zaa-modifier-native-cache-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
}

fn repeated(
    runner: &mut OrcaRunner,
    base: &[u8],
    case: Case,
    fixture_proof: &serde_json::Value,
) -> Vec<Recorded> {
    let (project, proof) = input::prepare(base, case);
    let mut runs = Vec::new();
    for repeat in 1..=2 {
        fresh_cache(runner);
        let native_root = runner
            .work
            .join(format!("zaa-modifier-{}-{repeat}", case.name()));
        fs::create_dir(&native_root).unwrap();
        let project_path = native_root.join("input.3mf");
        fs::write(&project_path, &project).unwrap();
        let output_dir = native_root.join("native");
        fs::create_dir(&output_dir).unwrap();
        let sliced = runner
            .rerun_reference(&ExportedCase {
                label: format!("zaa-modifier-slice-z/{}/{repeat}", case.name()),
                project: project.clone(),
                project_path,
                output_dir,
            })
            .unwrap();
        let observed = activity::observe(&sliced.reference);
        let outcome = parity::compare_case(&sliced);
        let root = outcome.artifacts.unwrap();
        let cli = crate::runner::application::cli_command()
            .arg("slice")
            .arg("-o")
            .arg(root.join("cli.gcode"))
            .arg(root.join("input.3mf"))
            .output()
            .unwrap();
        fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
        fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
        let api = fs::read(root.join("ares.gcode")).unwrap();
        let cli_bytes = fs::read(root.join("cli.gcode")).unwrap();
        let cli_difference =
            parity::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes)
                .err();
        let normalized = parity::golden::normalize_one_generator_line(
            &sliced.reference,
            parity::golden::GeneratorKind::Orca,
        )
        .unwrap();
        let native_sha = fixture::hash(&normalized);
        let ares_sha = fixture::hash(&api);
        fs::write(root.join("zaa-modifier-witness.json"), serde_json::to_vec_pretty(&json!({
            "case":case.name(),"repeat":repeat,"fixture":fixture_proof,"input":proof,
            "input_sha256":fixture::hash(&project),"native_sha256":fixture::hash(&sliced.reference),
            "native_normalized_sha256":native_sha,"native_eligible_sha256":observed.eligible_sha256,
            "native_xyz":observed.xyz,"native_modifier_xyz_count":observed.modifier_xyz,
            "ares_sha256":ares_sha,"cli_sha256":fixture::hash(&cli_bytes),"cli_exit":cli.status.code(),
            "status":outcome.status,"detail":outcome.detail,"cli_difference":cli_difference,
            "fresh_native_cache":runner.cache
        })).unwrap()).unwrap();
        assert!(
            std::str::from_utf8(&sliced.reference)
                .unwrap()
                .contains("; CONFIG_BLOCK_END"),
            "Native footer: {root:?}"
        );
        assert_eq!(
            !observed.xyz.is_empty(),
            case.active(),
            "Native ZAA activity: {root:?}"
        );
        if case.active() {
            assert!(
                observed.modifier_xyz > 0,
                "Native intersects modifier roof: {root:?}"
            );
        }
        if matches!(case, Case::ModifierOnly) {
            assert_eq!(
                observed.modifier_xyz,
                observed.xyz.len(),
                "Native stays in modifier: {root:?}"
            );
        }
        assert!(cli.status.success(), "CLI: {root:?}");
        assert_eq!(api, cli_bytes, "public API/CLI raw equality: {root:?}");
        runs.push(Recorded {
            root,
            status: outcome.status,
            detail: outcome.detail,
            native_sha,
            ares_sha,
            eligible_sha: observed.eligible_sha256,
            cli_difference,
        });
    }
    assert_eq!(
        runs[0].native_sha, runs[1].native_sha,
        "Native fresh-repeat complete output: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].ares_sha, runs[1].ares_sha,
        "Ares repeat raw output: {:?}",
        runs[0].root
    );
    runs
}

fn assert_complete(run: &Recorded) {
    assert_eq!(run.status, "PASS", "{}; {:?}", run.detail, run.root);
    assert!(
        run.cli_difference.is_none(),
        "CLI complete output: {:?}; {:?}",
        run.cli_difference,
        run.root
    );
}
