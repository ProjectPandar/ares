//! Fill.cpp:1631 / FillBase.cpp:311-313: regional object ironing direction.
//! Public failures: changed frozen input, absent Native Ironing activity,
//! unstable output, API/CLI drift, incorrect Ironing XY/E and complete-output drift.
use super::fixture;
use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner},
};
use serde_json::json;
use std::{fs, path::PathBuf};

#[path = "zaa_object_ironing_observe.rs"]
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
fn flag_false_preserves_ironing_and_complete_output() {
    verify(Case::Alternate);
}

#[test]
fn flag_true_preserves_ironing_and_complete_output() {
    verify(Case::NonAlternating);
}

#[test]
fn disabled_zaa_flag_true_preserves_ironing_and_complete_output() {
    verify(Case::DisabledZaa);
}

struct Recorded {
    root: PathBuf,
    native: Vec<[f64; 5]>,
    non_ironing: Vec<[f64; 8]>,
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
        Case::Alternate => "c5d72f90b67bb660a9db3e338aa5e2776213dd71cc0e2cbfb35a42504ec84d1d",
        Case::NonAlternating => "a835814e4d5f50ce32e7ddccd36d142ed8da928de51aa579ca492f74c6442274",
        Case::DisabledZaa => "cdecbac900db4d1982b05168f4cc63266dd83602a6075a12780f1e676984e53f",
    };
    let project = fs::read(
        parity::runner::repo_root()
            .join("tests/parity/zaa-object-ironing-direction")
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
        ares_core::ProcessInfillPattern::Concentric
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
    assert_eq!(
        loaded.settings().process.region.ironing_pattern,
        ares_core::ProcessInfillPattern::Rectilinear
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
        assert_eq!(
            runs[0].non_ironing, controls[0].non_ironing,
            "Native non-Ironing geometry unchanged"
        );
        assert_ne!(
            observe::geometry(&runs[0].native),
            observe::geometry(&controls[0].native),
            "Native layer12 undirected Ironing geometry activity: {:?}",
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
            .prefix("zaa-object-ironing-native-cache-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let native_root = tempfile::Builder::new()
            .prefix("zaa-object-ironing-input-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let project_path = native_root.join("input.3mf");
        fs::write(&project_path, &project).unwrap();
        let output_dir = native_root.join("native");
        fs::create_dir(&output_dir).unwrap();
        let sliced = runner
            .rerun_reference(&ExportedCase {
                label: format!("zaa-object-ironing-direction/{}/{repeat}", case.name()),
                project: project.clone(),
                project_path,
                output_dir,
            })
            .unwrap();
        let native = observe::ironing(&sliced.reference);
        let non_ironing = observe::non_ironing(&sliced.reference);
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
        let api = observe::ironing(&api_bytes);
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
        fs::write(root.join("zaa-object-ironing-witness.json"),serde_json::to_vec_pretty(&json!({
            "case":case.name(),"enabled":case.enabled(),"flag":case.flag(),"repeat":repeat,"layer":12,"input_sha256":fixture::hash(&project),
            "native_sha256":fixture::hash(&sliced.reference),"native_normalized_sha256":native_sha,
            "native_non_ironing":non_ironing,"native_ironing":native,"api_ironing":api,"native_ironing_sha256":fixture::hash(&serde_json::to_vec(&native).unwrap()),
            "api_ironing_sha256":fixture::hash(&serde_json::to_vec(&api).unwrap()),"api_sha256":api_sha,
            "cli_sha256":fixture::hash(&cli_bytes),"cli_exit":cli.status.code(),"status":outcome.status,
            "detail":outcome.detail,"cli_difference":cli_difference,"fresh_native_cache":runner.cache
        })).unwrap()).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        for line in [
            "; CONFIG_BLOCK_END".to_owned(),
            format!("; zaa_enabled = {}", u8::from(case.enabled())),
            "; sparse_infill_pattern = concentric".to_owned(),
            "; ironing_type = topmost".to_owned(),
            "; ironing_pattern = rectilinear".to_owned(),
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
            "object Rectilinear ironing fixture has no bridge: {root:?}"
        );
        assert!(
            !native.is_empty(),
            "Native layer12 Ironing active: {root:?}"
        );
        assert!(cli.status.success(), "CLI: {root:?}");
        assert_eq!(
            api_bytes, cli_bytes,
            "public API/CLI raw equality: {root:?}"
        );
        runs.push(Recorded {
            root,
            native,
            non_ironing,
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
        "Native fresh-repeat Ironing XY/E: {:?}",
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
        "layer12 object Ironing XY/E: {:?}",
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
