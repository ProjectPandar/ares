//! Fill.cpp:1235-1242/1446 -> FillBase.cpp:275-323: regional ZAA direction.
//! Public failures: inactive/rejected Native, unstable geometry, changed false
//! or disabled-ZAA controls, API/CLI disagreement and whole-output divergence.
use super::fixture::{self, Scope};
use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner},
};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command};

#[path = "zaa_fill_direction_input.rs"]
mod input;
use input::Case;

#[test]
fn flag_false_preserves_complete_output() {
    verify(Case::Alternate);
}

#[test]
fn flag_true_preserves_complete_output() {
    verify(Case::NonAlternating);
}

#[test]
fn disabled_zaa_flag_true_preserves_complete_output() {
    verify(Case::DisabledZaa);
}

struct Recorded {
    root: PathBuf,
    status: &'static str,
    detail: String,
    native_sha: String,
    ares_sha: String,
    solid_sha: String,
    cli_difference: Option<String>,
}

fn verify(case: Case) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    fresh_cache(&mut runner);
    let (exported, fixture_proof) = fixture::prepare(&runner, Scope::Object, "rectilinear", false);
    let controls = matches!(case, Case::NonAlternating).then(|| {
        repeated(
            &mut runner,
            &exported.project,
            Case::Alternate,
            &fixture_proof,
        )
    });
    if let Some(controls) = &controls {
        for run in controls {
            assert_complete(run);
        }
    }
    let runs = repeated(&mut runner, &exported.project, case, &fixture_proof);
    if let Some(controls) = controls {
        assert_ne!(
            runs[0].solid_sha, controls[0].solid_sha,
            "Native non-alternating internal-solid geometry: {:?}",
            runs[0].root
        );
    }
    for run in &runs {
        assert_complete(run);
    }
}

fn fresh_cache(runner: &mut OrcaRunner) {
    runner.cache = tempfile::Builder::new()
        .prefix("zaa-fill-direction-cache-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
}

fn repeated(
    runner: &mut OrcaRunner,
    base: &[u8],
    case: Case,
    fixture: &serde_json::Value,
) -> Vec<Recorded> {
    let (project, proof) = input::prepare(base, case);
    let mut runs = Vec::new();
    for repeat in 1..=2 {
        fresh_cache(runner);
        let native_root = runner
            .work
            .join(format!("zaa-direction-{}-{repeat}", case.name()));
        fs::create_dir(&native_root).unwrap();
        let project_path = native_root.join("input.3mf");
        fs::write(&project_path, &project).unwrap();
        let output_dir = native_root.join("native");
        fs::create_dir(&output_dir).unwrap();
        let sliced = runner
            .rerun_reference(&ExportedCase {
                label: format!("zaa-fill-direction/{}/{repeat}", case.name()),
                project: project.clone(),
                project_path,
                output_dir,
            })
            .unwrap();
        let (solid, xyz) = observe(&sliced.reference);
        let solid_sha = fixture::hash(&serde_json::to_vec(&solid).unwrap());
        let outcome = parity::compare_case(&sliced);
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
        fs::write(root.join("zaa-fill-direction-witness.json"), serde_json::to_vec_pretty(&json!({
            "case":case.name(),"repeat":repeat,"fixture":fixture,"input":proof,
            "input_sha256":fixture::hash(&project),"native_sha256":fixture::hash(&sliced.reference),
            "native_normalized_sha256":native_sha,"native_solid_sha256":solid_sha,
            "native_internal_solid":solid,"native_xyz_count":xyz,"ares_sha256":ares_sha,
            "cli_sha256":fixture::hash(&cli_bytes),"cli_exit":cli.status.code(),
            "status":outcome.status,"detail":outcome.detail,"cli_difference":cli_difference,
            "fresh_native_cache":runner.cache
        })).unwrap()).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        for line in [
            "; CONFIG_BLOCK_END".to_owned(),
            format!("; zaa_enabled = {}", u8::from(case.enabled())),
            format!(
                "; zaa_dont_alternate_fill_direction = {}",
                u8::from(case.flag())
            ),
        ] {
            assert!(
                reference.lines().any(|found| found == line),
                "Native footer/option {line}: {root:?}"
            );
        }
        assert!(
            !solid.is_empty(),
            "Native internal solid role active: {root:?}"
        );
        assert_eq!(xyz > 0, case.enabled(), "Native ZAA activity: {root:?}");
        assert!(cli.status.success(), "CLI: {root:?}");
        assert_eq!(api, cli_bytes, "public API/CLI raw output: {root:?}");
        runs.push(Recorded {
            root,
            status: outcome.status,
            detail: outcome.detail,
            native_sha,
            ares_sha,
            solid_sha,
            cli_difference,
        });
    }
    assert_eq!(
        runs[0].native_sha, runs[1].native_sha,
        "Native fresh-repeat output: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].ares_sha, runs[1].ares_sha,
        "Ares repeat raw output: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].solid_sha, runs[1].solid_sha,
        "Native repeat direction witness: {:?}",
        runs[0].root
    );
    runs
}

fn observe(bytes: &[u8]) -> (Vec<String>, usize) {
    let mut role = "";
    let mut solid = Vec::new();
    let mut xyz = 0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && (line.contains(" X") || line.contains(" Y"))
            && line
                .split_ascii_whitespace()
                .find_map(|token| token.strip_prefix('E'))
                .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
        {
            if role == "Internal solid infill" {
                solid.push(line.to_owned());
            }
            xyz += usize::from(line.contains(" Z"));
        }
    }
    (solid, xyz)
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
