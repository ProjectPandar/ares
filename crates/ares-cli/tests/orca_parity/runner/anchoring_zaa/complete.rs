//! Shared fresh Native/public API/CLI complete-output recording.

use std::{fs, path::PathBuf, process::Command};

use crate::runner::{ExportedCase, OrcaRunner};
use serde_json::json;

use super::observe;

pub(super) struct Recorded {
    pub(super) root: PathBuf,
    pub(super) native_sha: String,
    pub(super) api_sha: String,
    pub(super) status: &'static str,
    pub(super) detail: String,
    pub(super) cli_difference: Option<String>,
    pub(super) native_sparse: Vec<(usize, String)>,
    pub(super) api_sparse: Vec<(usize, String)>,
    pub(super) native: Vec<u8>,
    pub(super) api_bytes: Vec<u8>,
    pub(super) cli_bytes: Vec<u8>,
    pub(super) cli_exit: Option<i32>,
}

pub(super) fn assert_complete(run: &Recorded) {
    assert_eq!(run.status, "PASS", "{}; {:?}", run.detail, run.root);
    assert!(
        run.cli_difference.is_none(),
        "CLI strict complete output: {:?}; {:?}",
        run.cli_difference,
        run.root
    );
}

pub(super) fn assert_repeated(runs: &[Recorded]) {
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
}

pub(super) fn run_once(
    runner: &mut OrcaRunner,
    label: &str,
    case: &str,
    repeat: usize,
    project: &[u8],
) -> Recorded {
    runner.cache = tempfile::Builder::new()
        .prefix(&format!("{label}-cache-"))
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let native_root = tempfile::Builder::new()
        .prefix(&format!("{label}-input-"))
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let project_path = native_root.join("input.3mf");
    fs::write(&project_path, project).unwrap();
    let output_dir = native_root.join("native");
    fs::create_dir(&output_dir).unwrap();
    let sliced = runner
        .rerun_reference(&ExportedCase {
            label: format!("{label}/{case}/{repeat}"),
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
    fs::write(
        root.join("complete-output-witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case, "repeat": repeat, "input_sha256": observe::hash(project),
            "native_normalized_sha256": native_sha, "api_sha256": api_sha,
            "cli_sha256": observe::hash(&cli_bytes), "cli_exit": cli.status.code(),
            "api_cli_equal": api_bytes == cli_bytes,
            "fresh_native_cache": runner.cache, "status": outcome.status,
            "detail": outcome.detail, "cli_difference": cli_difference
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "CLI execution: {root:?}");
    assert_eq!(api_bytes, cli_bytes, "public API/CLI raw bytes: {root:?}");
    Recorded {
        root,
        native_sha,
        api_sha,
        status: outcome.status,
        detail: outcome.detail,
        cli_difference,
        native_sparse,
        api_sparse,
        native: sliced.reference,
        api_bytes,
        cli_bytes,
        cli_exit: cli.status.code(),
    }
}
