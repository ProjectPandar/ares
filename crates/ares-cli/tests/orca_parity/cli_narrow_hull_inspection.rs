#![cfg(target_os = "linux")]

use std::{fs, process::Command};

use super::cli_model_inspection::{compare, inventory, open_box, run_native};
use crate::runner::OrcaRunner;

#[test]
fn mesh_inspection_reports_planes_from_a_narrow_hull() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("narrow-hull-inspection-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("narrow-fan.stl");
    fs::write(
        &input,
        include_bytes!("../../../../tests/parity/qt-narrow-hull/narrow-fan.stl"),
    )
    .unwrap();
    let args = ["--inspect-mesh", "--arrange", "0"];
    let native = run_native(&runner, &root.join("native"), &args, input.to_str());
    assert_eq!(native.0, Some(0), "Native: {root:?}");
    assert_eq!(
        native,
        run_native(&runner, &root.join("native-replay"), &args, input.to_str()),
        "Native replay: {root:?}"
    );
    // The pinned C++ wrapper can report Qhull's narrow-input diagnostic even
    // when inspection succeeds. The JSON output must still match exactly.
    let diagnostics = String::from_utf8_lossy(&native.2);
    assert!(
        diagnostics.is_empty() || diagnostics.contains("initial hull is narrow"),
        "unexpected Native diagnostics: {root:?}"
    );
    let work = root.join("ares");
    fs::create_dir(&work).unwrap();
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(&work)
        .args(["slice", "--inspect-mesh", "--outputdir", "."])
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("ares.stdout"), &ares.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &ares.stderr).unwrap();
    assert_eq!(ares.status.code(), native.0, "exit: {root:?}");
    assert!(ares.stdout == native.1, "stdout: {root:?}");
    assert!(ares.stderr.is_empty(), "Ares diagnostics: {root:?}");
    assert_eq!(inventory(&work), native.3, "outputs: {root:?}");
}

#[test]
fn mesh_inspection_reports_planes_from_an_ordinary_hull() {
    compare(&["--inspect-mesh"], Some(("open-box.stl", open_box())), 0);
}
