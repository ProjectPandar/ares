//! Fill.cpp:1440-1447 / FillBase.cpp:311-313: regional sparse-anchor direction.
//! Public failures: changed frozen input, absent Native bridge activity,
//! unstable output, API/CLI drift, incorrect bridge XY/E and complete-output drift.
use super::fixture;
use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner},
};
use serde_json::json;
use std::{fs, path::PathBuf};

#[path = "zaa_anchoring_observe.rs"]
mod observe;

#[test]
fn flag0_preserves_internal_bridge_and_complete_output() {
    verify(false);
}

#[test]
fn flag1_preserves_internal_bridge_and_complete_output() {
    verify(true);
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

fn frozen(flag: bool) -> Vec<u8> {
    let (file, expected) = if flag {
        (
            "flag1.3mf",
            "6ab88a9a3d5ae45bfadff52e6ff0e85e0b16f8b3b761255454f40134036e0a7e",
        )
    } else {
        (
            "flag0.3mf",
            "9c724601a69ae4743e4caa9594baa6f7cef131823f84a5eb035592ab3b20c219",
        )
    };
    let project = fs::read(
        parity::runner::repo_root()
            .join("tests/parity/zaa-anchoring-direction")
            .join(file),
    )
    .unwrap();
    assert_eq!(fixture::hash(&project), expected, "frozen input {file}");
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects().first().unwrap().instances().len(), 1);
    assert!(loaded.settings().process.region.zaa_enabled.0);
    assert_eq!(
        loaded
            .settings()
            .process
            .region
            .zaa_dont_alternate_fill_direction
            .0,
        flag
    );
    project
}

fn verify(flag: bool) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let controls = flag.then(|| repeated(&mut runner, false));
    if let Some(controls) = &controls {
        for run in controls {
            assert_parity(run);
        }
    }
    let runs = repeated(&mut runner, flag);
    if let Some(controls) = controls {
        assert_ne!(
            runs[0].native, controls[0].native,
            "Native layer47 Internal Bridge activity: {:?}",
            runs[0].root
        );
    }
    for run in &runs {
        assert_parity(run);
    }
}

fn repeated(runner: &mut OrcaRunner, flag: bool) -> Vec<Recorded> {
    let project = frozen(flag);
    let mut runs = Vec::new();
    for repeat in 1..=2 {
        runner.cache = tempfile::Builder::new()
            .prefix("zaa-anchoring-native-cache-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let native_root = tempfile::Builder::new()
            .prefix("zaa-anchoring-input-")
            .tempdir_in(&runner.work)
            .unwrap()
            .keep();
        let project_path = native_root.join("input.3mf");
        fs::write(&project_path, &project).unwrap();
        let output_dir = native_root.join("native");
        fs::create_dir(&output_dir).unwrap();
        let sliced = runner
            .rerun_reference(&ExportedCase {
                label: format!("zaa-anchoring-direction/flag-{}/{repeat}", u8::from(flag)),
                project: project.clone(),
                project_path,
                output_dir,
            })
            .unwrap();
        let native = observe::bridge(&sliced.reference);
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
        let api = observe::bridge(&api_bytes);
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
        fs::write(root.join("zaa-anchoring-witness.json"),serde_json::to_vec_pretty(&json!({
            "flag":flag,"repeat":repeat,"layer":47,"input_sha256":fixture::hash(&project),
            "native_sha256":fixture::hash(&sliced.reference),"native_normalized_sha256":native_sha,
            "native_bridge":native,"api_bridge":api,"native_bridge_sha256":fixture::hash(&serde_json::to_vec(&native).unwrap()),
            "api_bridge_sha256":fixture::hash(&serde_json::to_vec(&api).unwrap()),"api_sha256":api_sha,
            "cli_sha256":fixture::hash(&cli_bytes),"cli_exit":cli.status.code(),"status":outcome.status,
            "detail":outcome.detail,"cli_difference":cli_difference,"fresh_native_cache":runner.cache
        })).unwrap()).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        for line in [
            "; CONFIG_BLOCK_END".to_owned(),
            "; zaa_enabled = 1".to_owned(),
            format!("; zaa_dont_alternate_fill_direction = {}", u8::from(flag)),
        ] {
            assert!(
                reference.lines().any(|found| found == line),
                "Native footer/flag: {root:?}"
            );
        }
        assert!(
            !native.is_empty(),
            "Native layer47 Internal Bridge active: {root:?}"
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
        "Native fresh-repeat bridge XY/E: {:?}",
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
        "layer47 Internal Bridge anchoring XY/E: {:?}",
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
