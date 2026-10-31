//! FillConcentric.cpp:92-102 leaves short-line parameters indeterminate.
//! ARD-0069 defines initialized authority; preserve all seven SAFC contracts.
use super::{ExportedCase, OrcaRunner, repo_root};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path, path::PathBuf, process::Command};

pub(crate) const REQUEST: &str = "ARES_SAFC_CONCENTRIC_REQUEST";
const CHILD: &str = "option_coverage::safc_coverage::authority_tests::safc_concentric_cases_use_initialized_authority_in_both_modes";

#[derive(Serialize, Deserialize)]
pub(crate) struct References {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) producer: Value,
}

pub(crate) fn producer(runner: &OrcaRunner) -> Value {
    runner.producer.get().unwrap().receipt.clone()
}

pub(crate) fn initialized_pair(cases: &[&ExportedCase], root: &Path) -> Option<References> {
    let endpoint = crate::native_environment::binary("ARES_KSR_INITIALIZED_BIN")?;
    let native = crate::native_environment::binary("ARES_KSR_INITIALIZED_NATIVE_BIN")?;
    let resources = crate::native_environment::value("ARES_KSR_INITIALIZED_RESOURCES_DIR")?;
    let directory = tempfile::Builder::new()
        .prefix("safc-initialized-pair-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let mut inputs = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let input_dir = directory.join(index.to_string());
        fs::create_dir(&input_dir).unwrap();
        let input = input_dir.join("input.3mf");
        fs::write(&input, &case.project).unwrap();
        inputs.push(input);
    }
    let request = directory.join("request.json");
    fs::write(&request, serde_json::to_vec_pretty(&inputs).unwrap()).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--nocapture"])
        .env(REQUEST, &request)
        .env(
            "ARES_ORCA_BIN",
            repo_root().join("scripts/orca-seeded-parity.sh"),
        )
        .env("ARES_ORCA_NATIVE_BIN", &native)
        .env("ARES_ORCA_RESOURCES_DIR", &resources)
        .env("ORCA_REPLAY_BIN", &endpoint)
        .env("ORCA_REPLAY_SEED", "0")
        .env("ARES_PARITY_SEEDED_REFERENCE", "1")
        .env_remove("ORCA_APPDIR")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    fs::write(directory.join("child.stdout"), &output.stdout).unwrap();
    fs::write(directory.join("child.stderr"), &output.stderr).unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{directory:?}: {diagnostic}");
    if diagnostic.contains("SKIPPED:") {
        assert!(
            !crate::native_environment::required(),
            "{directory:?}: {diagnostic}"
        );
        crate::native_environment::skipped(&diagnostic);
        return None;
    }
    let references: References =
        serde_json::from_slice(&fs::read(directory.join("response.json")).unwrap()).unwrap();
    assert_eq!(references.paths.len(), cases.len(), "{directory:?}");
    Some(references)
}

pub(crate) fn child(request: &Path) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let inputs: Vec<PathBuf> = serde_json::from_slice(&fs::read(request).unwrap()).unwrap();
    let directory = request.parent().unwrap();
    let mut paths = Vec::new();
    for (index, input) in inputs.into_iter().enumerate() {
        let output_dir = runner.work.join(format!("native-{index}"));
        fs::create_dir(&output_dir).unwrap();
        let reference = runner
            .slice_case(&ExportedCase {
                label: format!("safc/initialized/{index}"),
                project: fs::read(&input).unwrap(),
                project_path: input,
                output_dir,
            })
            .unwrap()
            .reference;
        let path = directory.join(format!("reference-{index}.gcode"));
        fs::write(&path, reference).unwrap();
        paths.push(path);
    }
    let response = References {
        paths,
        producer: producer(&runner),
    };
    fs::write(
        directory.join("response.json"),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
}
