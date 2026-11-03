//! Fill.cpp:1235-1242 / FillBase.cpp:311-313: regional Honeycomb direction.
//! Public failures: changed frozen input, absent Native sparse activity,
//! unstable output, API/CLI drift, incorrect sparse XY/E and complete-output drift.
use super::fixture;
use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner},
};
use serde_json::json;
use std::{fs, path::PathBuf};

#[path = "zaa_honeycomb_observe.rs"]
mod observe;

#[derive(Clone, Copy)]
enum Case {
    Alternate,
    NonAlternating,
    DisabledZaa,
}

impl Case {
    fn enabled(self) -> bool {
        !matches!(self, Self::DisabledZaa)
    }
    fn flag(self) -> bool {
        !matches!(self, Self::Alternate)
    }
    fn name(self) -> &'static str {
        match self {
            Self::Alternate => "flag-false",
            Self::NonAlternating => "flag-true",
            Self::DisabledZaa => "disabled-zaa-flag-true",
        }
    }
}

#[test]
fn flag_false_preserves_sparse_and_complete_output() {
    verify(Case::Alternate);
}

#[test]
fn flag_true_preserves_sparse_and_complete_output() {
    verify(Case::NonAlternating);
}

#[test]
fn disabled_zaa_flag_true_preserves_sparse_and_complete_output() {
    verify(Case::DisabledZaa);
}

struct Recorded {
    root: PathBuf,
    native: Vec<[f64; 5]>,
    api: Vec<[f64; 5]>,
    native_sha: String,
    api_sha: String,
    status: &'static str,
    detail: String,
    cli_difference: Option<String>,
}

fn frozen(case: Case) -> Vec<u8> {
    let file = format!("{}.3mf", case.name());
    let expected = match case {
        Case::Alternate => "27299db2211226b748467cb0978951ce7c50b69065d7ceea5fe020783774b0f8",
        Case::NonAlternating => "0b62b14e539bda8537af5ba86c0695d99e2720bfad491431dbfd3a6670681c6a",
        Case::DisabledZaa => "d88cb59ce2a810545e8de2a1e5ba890227305485f4bdd5501a54bda049636f58",
    };
    let project = fs::read(
        parity::runner::repo_root()
            .join("tests/parity/zaa-honeycomb-direction")
            .join(&file),
    )
    .unwrap();
    assert_eq!(fixture::hash(&project), expected, "frozen input {file}");
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects().first().unwrap().instances().len(), 1);
    assert_eq!(
        loaded.settings().process.region.zaa_enabled.0,
        case.enabled()
    );
    assert_eq!(
        loaded.settings().process.region.sparse_infill_pattern,
        ares_core::ProcessInfillPattern::Honeycomb
    );
    assert_eq!(
        loaded
            .settings()
            .process
            .region
            .zaa_dont_alternate_fill_direction
            .0,
        case.flag()
    );
    project
}

fn verify(case: Case) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let controls =
        matches!(case, Case::NonAlternating).then(|| repeated(&mut runner, Case::Alternate));
    if let Some(controls) = &controls {
        for run in controls {
            assert_parity(run);
        }
    }
    let runs = repeated(&mut runner, case);
    if let Some(controls) = controls {
        assert_ne!(
            observe::geometry(&runs[0].native),
            observe::geometry(&controls[0].native),
            "Native layer2 undirected Sparse geometry activity: {:?}",
            runs[0].root
        );
    }
    for run in &runs {
        assert_parity(run);
    }
}

fn repeated(runner: &mut OrcaRunner, case: Case) -> Vec<Recorded> {
    let project = frozen(case);
    let mut runs = Vec::new();
    for repeat in 1..=2 {
        runner.cache = tempfile::Builder::new()
            .prefix("zaa-honeycomb-native-cache-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let native_root = tempfile::Builder::new()
            .prefix("zaa-honeycomb-input-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let project_path = native_root.join("input.3mf");
        fs::write(&project_path, &project).unwrap();
        let output_dir = native_root.join("native");
        fs::create_dir(&output_dir).unwrap();
        let sliced = runner
            .rerun_reference(&ExportedCase {
                label: format!("zaa-honeycomb-direction/{}/{repeat}", case.name()),
                project: project.clone(),
                project_path,
                output_dir,
            })
            .unwrap();
        let native = observe::sparse(&sliced.reference);
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
        let api_bytes = fs::read(root.join("ares.gcode")).unwrap();
        let cli_bytes = fs::read(root.join("cli.gcode")).unwrap();
        let api = observe::sparse(&api_bytes);
        let cli_difference =
            parity::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes)
                .err();
        let normalized = parity::golden::normalize_one_generator_line(
            &sliced.reference,
            parity::golden::GeneratorKind::Orca,
        )
        .unwrap();
        let native_sha = fixture::hash(&normalized);
        let api_sha = fixture::hash(&api_bytes);
        fs::write(root.join("zaa-honeycomb-witness.json"),serde_json::to_vec_pretty(&json!({
            "case":case.name(),"enabled":case.enabled(),"flag":case.flag(),"repeat":repeat,"layer":2,"input_sha256":fixture::hash(&project),
            "native_sha256":fixture::hash(&sliced.reference),"native_normalized_sha256":native_sha,
            "native_sparse":native,"api_sparse":api,"native_sparse_sha256":fixture::hash(&serde_json::to_vec(&native).unwrap()),
            "api_sparse_sha256":fixture::hash(&serde_json::to_vec(&api).unwrap()),"api_sha256":api_sha,
            "cli_sha256":fixture::hash(&cli_bytes),"cli_exit":cli.status.code(),"status":outcome.status,
            "detail":outcome.detail,"cli_difference":cli_difference,"fresh_native_cache":runner.cache
        })).unwrap()).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        for line in [
            "; CONFIG_BLOCK_END".to_owned(),
            format!("; zaa_enabled = {}", u8::from(case.enabled())),
            "; sparse_infill_pattern = honeycomb".to_owned(),
            format!(
                "; zaa_dont_alternate_fill_direction = {}",
                u8::from(case.flag())
            ),
        ] {
            assert!(
                reference.lines().any(|found| found == line),
                "Native footer/flag: {root:?}"
            );
        }
        assert!(
            !reference
                .lines()
                .any(|line| line == ";TYPE:Internal Bridge"),
            "ordinary Honeycomb fixture has no bridge: {root:?}"
        );
        assert!(
            !native.is_empty(),
            "Native layer2 Sparse infill active: {root:?}"
        );
        assert!(cli.status.success(), "CLI: {root:?}");
        assert_eq!(
            api_bytes, cli_bytes,
            "public API/CLI raw equality: {root:?}"
        );
        runs.push(Recorded {
            root,
            native,
            api,
            native_sha,
            api_sha,
            status: outcome.status,
            detail: outcome.detail,
            cli_difference,
        });
    }
    assert_eq!(
        runs[0].native_sha, runs[1].native_sha,
        "Native fresh-repeat complete output: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].native, runs[1].native,
        "Native fresh-repeat sparse XY/E: {:?}",
        runs[0].root
    );
    assert_eq!(
        runs[0].api_sha, runs[1].api_sha,
        "API repeat raw output: {:?}",
        runs[0].root
    );
    runs
}

fn assert_parity(run: &Recorded) {
    assert_eq!(
        run.api, run.native,
        "layer2 Honeycomb Sparse infill XY/E: {:?}",
        run.root
    );
    assert_eq!(run.status, "PASS", "{}; {:?}", run.detail, run.root);
    assert!(
        run.cli_difference.is_none(),
        "CLI strict complete output: {:?}; {:?}",
        run.cli_difference,
        run.root
    );
}
