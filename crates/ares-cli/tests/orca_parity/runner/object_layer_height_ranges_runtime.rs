use std::{fs, process::Command};

#[test]
fn optional_missing_native_never_consumes_frozen_range_reference() {
    let artifacts = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        tempfile::Builder::new()
            .prefix("object-range-height-policy-")
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
            "runner::object_layer_height_ranges::object_range_layer_height_changes_layer_schedule_and_overhang_seam",
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
    assert!(diagnostic.contains("SKIPPED"), "{diagnostic}");
    for entry in fs::read_dir(&child_root).unwrap() {
        let directory = entry.unwrap().path();
        for filename in ["native.gcode", "result.json"] {
            assert!(
                !directory.join(filename).exists(),
                "missing Native consumed frozen reference: {directory:?}/{filename}"
            );
        }
    }
}
