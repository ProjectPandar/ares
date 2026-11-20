#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::cli_no_check::{inventory, run};
use crate::{
    artifacts, native_environment, native_wall_clock::result_comparison_bytes, runner::OrcaRunner,
};

#[test]
fn cli_checked_slice_rejects_empty_layers_before_export() {
    compare(&["--slice", "1"], false);
}

#[test]
fn cli_no_check_before_slice_exports_with_the_empty_layer_warning() {
    compare(&["--no-check", "--slice", "1"], true);
}

#[test]
fn cli_no_check_after_slice_still_rejects_empty_layers() {
    compare(&["--slice", "1", "--no-check"], false);
}

fn compare(actions: &[&str], no_check: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        native_environment::skipped("CLI empty-layer Native unavailable");
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-empty-layer-warning-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("odd-rotated.3mf");
    fs::write(
        &input,
        include_bytes!("../../../../tests/parity/mesh-edge-pairing/odd-rotated.3mf"),
    )
    .unwrap();
    let mut argv: Vec<OsString> = ["--debug", "2", "--outputdir", "out"]
        .into_iter()
        .chain(actions.iter().copied())
        .map(OsString::from)
        .collect();
    argv.push(input.into());
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let report: Value = serde_json::from_slice(&native_report).unwrap();

    assert_eq!(
        native.status.code(),
        Some(if no_check { 0 } else { 156 }),
        "{root:?}"
    );
    assert_eq!(
        report["return_code"],
        if no_check { 0 } else { -100 },
        "{root:?}"
    );
    let plate = &report["sliced_plates"][0];
    assert_eq!(
        plate["triangle_count"],
        if no_check { 13 } else { 0 },
        "{root:?}"
    );
    let message = plate["warning_message"].as_str().unwrap();
    assert!(
        message.starts_with("The object has empty layers between "),
        "{root:?}"
    );
    assert!(native.stderr.is_empty(), "{root:?}");
    assert_eq!(
        native_cwd.join("out/plate_1.gcode").is_file(),
        no_check,
        "{root:?}"
    );

    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
    let ares_report = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&ares_report) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );

    let record = format!(
        "plate 1: found slicing warnings: {message}, no_check={}\n",
        u8::from(no_check)
    );
    for cwd in [&native_cwd, &ares_cwd] {
        let log = fs::read_to_string(cwd.join("00000.log")).unwrap();
        assert_eq!(log.matches(&record).count(), 1, "{cwd:?}");
    }
}
