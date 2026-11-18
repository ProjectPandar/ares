//! Actual caller entrypoints, stopped at initialization; no Orca case executes.
use std::{path::Path, process::Command};

fn invoke(selector: &str, root: Option<&Path>, cwd: &Path) -> std::process::Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", selector, "--nocapture"])
        .current_dir(cwd)
        .env("ARES_ORCA_BIN", std::env::current_exe().unwrap())
        .env_remove("ARES_REQUIRE_ORCA")
        .env("ARES_PARITY_SWEEP", "1")
        .env("ARES_PARITY_OPTIONS", "1")
        .env_remove("CLUSTER_DUMP")
        .env_remove("ARES_PARITY_ARTIFACT_ROOT");
    if let Some(root) = root {
        command.env("ARES_PARITY_ARTIFACT_ROOT", root);
    }
    let output = command.output().unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    output
}

#[test]
fn configured_callers_fail_on_artifact_initialization_errors() {
    let temp = tempfile::tempdir().unwrap();
    let blocked = temp.path().join("blocked");
    std::fs::write(&blocked, b"not a directory").unwrap();
    let repo = super::repo_root();
    let relative = std::path::PathBuf::from("relative-artifacts");
    let mut failures = Vec::new();
    for selector in [
        "smoke::ender3_smoke",
        "smoke::top_concentric_smoke",
        "smoke::printer_sweep",
        "smoke::nearest_cluster_check",
        "smoke::prusa_core_one_dump",
    ] {
        for (name, root, cause) in [
            ("missing", None, "ARES_PARITY_ARTIFACT_ROOT"),
            ("relative", Some(relative.as_path()), "absolute"),
            ("repo-local", Some(repo.as_path()), "outside the repository"),
            ("unwritable", Some(blocked.as_path()), "blocked"),
        ] {
            let output = invoke(selector, root, temp.path());
            let text = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("{selector}/{name}: {}\n{text}", output.status);
            if output.status.success()
                || !text.contains(cause)
                || text.contains("skipping: no OrcaSlicer")
            {
                failures.push(format!("{selector}/{name}: {}", output.status));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "initialization falsely skipped or lost cause: {failures:?}"
    );
}
