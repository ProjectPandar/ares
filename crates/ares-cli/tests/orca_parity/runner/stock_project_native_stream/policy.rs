use std::{fs, process::Command};

#[test]
fn optional_missing_native_never_consumes_frozen_stock_project_reference() {
    missing_native(
        "runner::stock_project_native_stream::project_matches_initialized_orca_ordered_bytes",
    );
}

#[test]
fn optional_missing_native_never_consumes_frozen_stock_project_config_block() {
    missing_native(
        "runner::stock_project_native_stream::config_block::config_export_fixture_matches_resolved_views_byte_for_byte",
    );
}

#[test]
fn optional_missing_native_never_consumes_frozen_classic_records() {
    missing_native(
        "runner::stock_project_native_stream::classic::gcode_writer_and_do_export_real_records_reject_33_mutations",
    );
}

#[test]
fn optional_missing_native_never_consumes_frozen_browser_reference() {
    missing_native(
        "runner::stock_project_native_stream::browser::browser_output_matches_initialized_orca_ordered_bytes",
    );
}

#[test]
fn required_without_browser_capture_is_visibly_skipped() {
    let artifacts = match std::env::var_os("ARES_PARITY_ARTIFACT_ROOT") {
        Some(_) => tempfile::Builder::new()
            .prefix("ksr-browser-capture-policy-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap()),
        None => tempfile::tempdir(),
    }
    .unwrap()
    .keep();
    let child_root = artifacts.join("child");
    fs::create_dir(&child_root).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::stock_project_native_stream::browser::browser_output_matches_initialized_orca_ordered_bytes",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_BROWSER_GCODE")
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
    assert!(
        diagnostic.contains("running 1 test"),
        "{artifacts:?}: {diagnostic}"
    );
    assert!(output.status.success(), "{artifacts:?}: {diagnostic}");
    assert!(
        diagnostic.contains("SKIPPED: browser capture absent; run via web/e2e/ksrProject.spec.ts"),
        "{artifacts:?}: {diagnostic}"
    );
    assert_eq!(
        fs::read_dir(&child_root).unwrap().count(),
        0,
        "{artifacts:?}"
    );
}

fn missing_native(test: &str) {
    let artifacts = match std::env::var_os("ARES_PARITY_ARTIFACT_ROOT") {
        Some(_) => tempfile::Builder::new()
            .prefix("ksr-reference-policy-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap()),
        None => tempfile::tempdir(),
    }
    .unwrap()
    .keep();
    let child_root = artifacts.join("child");
    fs::create_dir(&child_root).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "0")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .env_remove("ARES_ORCA_BIN")
        .env_remove("ARES_ORCA_RESOURCES_DIR")
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
    assert!(
        diagnostic.contains("running 1 test"),
        "{artifacts:?}: {diagnostic}"
    );
    assert!(output.status.success(), "{artifacts:?}: {diagnostic}");
    assert!(
        diagnostic.contains("SKIPPED:"),
        "{artifacts:?}: {diagnostic}"
    );
    assert!(
        diagnostic.contains("Native comparison not executed"),
        "{artifacts:?}: {diagnostic}"
    );
    assert!(
        !diagnostic.contains("complete KSR artifacts:"),
        "missing Native fabricated KSR comparison artifacts: {artifacts:?}: {diagnostic}"
    );
    assert_eq!(
        fs::read_dir(&child_root).unwrap().count(),
        0,
        "{artifacts:?}"
    );
}
