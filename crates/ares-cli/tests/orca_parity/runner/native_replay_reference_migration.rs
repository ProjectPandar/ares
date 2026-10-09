use std::{fs, process::Command};

// GCode.cpp:2030,2461 exports the complete stream; slice_case retains the
// producer-bound receipt while the legacy replay manifest remains unchanged.
pub(crate) fn reference(source: &str) -> Option<Vec<u8>> {
    use crate::runner::{ExportedCase, OrcaRunner, repo_root};

    let runner = OrcaRunner::from_env().unwrap()?;
    let source = repo_root().join(source);
    let project = fs::read(&source).unwrap();
    let project_path = runner.work.join(source.file_name().unwrap());
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: format!("replay/{}", source.file_name().unwrap().to_str().unwrap()),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    Some(case.reference)
}

#[test]
fn optional_missing_native_never_fabricates_replay_comparison() {
    let artifacts = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        tempfile::Builder::new()
            .prefix("replay-reference-policy-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        tempfile::tempdir()
    }
    .unwrap()
    .keep();
    let child_root = artifacts.join("child");
    fs::create_dir(&child_root).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "replay_tests::unchanged_classic_replays_byte_identical_with_complete_paired_bytes_and_hashes",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "0")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .env("ARES_PARITY_ARTIFACT_ROOT", &child_root)
        .output()
        .unwrap();
    fs::write(artifacts.join("child.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("child.stderr"), &output.stderr).unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostic.contains("running 1 test"), "{diagnostic}");
    assert!(output.status.success(), "{diagnostic}");
    assert_eq!(
        fs::read_dir(&child_root).unwrap().count(),
        0,
        "missing Native fabricated replay comparison artifacts: {child_root:?}"
    );
    assert!(diagnostic.contains("SKIPPED:"), "{diagnostic}");
    assert!(
        diagnostic.contains("Native comparison not executed"),
        "{diagnostic}"
    );
}
