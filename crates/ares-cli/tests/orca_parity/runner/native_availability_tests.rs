//! Process-boundary environment-policy checks; these do not claim Native slicing parity.
use std::{path::Path, process::Command};

fn invoke(selector: &str, root: &Path, missing: &str, required: bool) -> std::process::Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", selector, "--nocapture"])
        .env("ARES_NATIVE_POLICY_CHILD", "1")
        .env("ARES_REQUIRE_ORCA", if required { "1" } else { "0" })
        .env("ARES_ORCA_BIN", std::env::current_exe().unwrap())
        .env("ARES_ORCA_NATIVE_BIN", std::env::current_exe().unwrap())
        .env("ARES_ORCA_RESOURCES_DIR", root)
        .env("ARES_PARITY_ARTIFACT_ROOT", root)
        .env_remove("ARES_PARITY_SWEEP")
        .env_remove("ARES_PARITY_OPTIONS")
        .env_remove("ARES_PARITY_HONEYCOMB_MULTILINE")
        .env_remove("ORCA_REPLAY_BIN")
        .env_remove(missing);
    let output = command.output().unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    output
}

fn text(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn missing_declaration_fails(key: &str) {
    let root = tempfile::tempdir().unwrap();
    let output = invoke(
        "runner::native_availability_tests::native_availability_entrypoint_probe",
        root.path(),
        key,
        true,
    );
    let diagnostic = text(&output);
    assert!(
        !output.status.success() && diagnostic.contains(key) && !diagnostic.contains("SKIPPED:"),
        "missing {key} falsely passed or lost cause: {}\n{diagnostic}",
        output.status
    );
}

#[test]
fn required_native_missing_binary_fails_at_initialization() {
    missing_declaration_fails("ARES_ORCA_NATIVE_BIN");
}

#[test]
fn required_native_missing_resources_fail_at_initialization() {
    missing_declaration_fails("ARES_ORCA_RESOURCES_DIR");
}

#[test]
fn required_native_missing_artifact_root_fails_at_initialization() {
    missing_declaration_fails("ARES_PARITY_ARTIFACT_ROOT");
}

#[test]
fn optional_native_absence_explicitly_reports_unexecuted_comparison() {
    let root = tempfile::tempdir().unwrap();
    let output = invoke(
        "smoke::ender3_smoke",
        root.path(),
        "ARES_PARITY_ARTIFACT_ROOT",
        false,
    );
    let diagnostic = text(&output);
    assert!(output.status.success(), "{diagnostic}");
    assert!(
        diagnostic.contains("SKIPPED:")
            && diagnostic.contains("ARES_PARITY_ARTIFACT_ROOT")
            && diagnostic.contains("not executed"),
        "optional comparison omission invisible or falsely labeled parity: {diagnostic}"
    );
}

#[test]
fn required_native_absence_cannot_hide_behind_disabled_selector() {
    let root = tempfile::tempdir().unwrap();
    let output = invoke(
        "honeycomb_multiline::honeycomb_multiline_ordered_bytes",
        root.path(),
        "ARES_ORCA_NATIVE_BIN",
        true,
    );
    let diagnostic = text(&output);
    assert!(
        diagnostic.contains("running 1 test"),
        "selector drift: {diagnostic}"
    );
    assert!(
        !output.status.success() && diagnostic.contains("ARES_ORCA_NATIVE_BIN"),
        "disabled selector concealed required Native absence: {diagnostic}"
    );
}

#[test]
fn required_procgcode_absence_fails_actual_vg1_entrypoint() {
    let root = tempfile::tempdir().unwrap();
    let output = invoke(
        "vg1_flush_state::vg1_x1c_virtual_state_and_vertex_effects_match_native",
        root.path(),
        "ARES_PROCGCODE_BIN",
        true,
    );
    let diagnostic = text(&output);
    assert!(
        diagnostic.contains("running 1 test"),
        "selector drift: {diagnostic}"
    );
    assert!(
        !output.status.success() && diagnostic.contains("ARES_PROCGCODE_BIN"),
        "missing procgcode was falsely accepted: {diagnostic}"
    );
}

#[test]
fn native_availability_entrypoint_probe() {
    if std::env::var("ARES_NATIVE_POLICY_CHILD").as_deref() != Ok("1") {
        return;
    }
    // Calls the actual initialization interface, no stubbed policy/Native execution.
    let _runner = super::OrcaRunner::from_env().unwrap();
}
