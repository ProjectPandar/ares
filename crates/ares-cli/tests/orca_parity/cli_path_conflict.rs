#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::cli_no_check::{inventory, run};
use crate::{
    artifacts, native_environment, native_wall_clock::result_comparison_bytes, runner::OrcaRunner,
};

#[test]
fn cli_path_conflict_takes_precedence_over_empty_layer_warnings() {
    compare(&["--slice", "1"], false);
}

#[test]
fn cli_no_check_before_slice_skips_conflicts_and_logs_each_critical_warning() {
    compare(&["--no-check", "--slice", "1"], true);
}

#[test]
fn cli_no_check_after_slice_still_rejects_path_conflicts() {
    compare(&["--slice", "1", "--no-check"], false);
}

fn compare(actions: &[&str], no_check: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        native_environment::skipped("CLI path conflict Native unavailable");
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-path-conflict-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("crossing-gaps.3mf");
    fs::write(
        &input,
        include_bytes!("../../../../tests/parity/mesh-edge-pairing/crossing-gaps.3mf"),
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
        Some(if no_check { 0 } else { 155 }),
        "{root:?}"
    );
    assert_eq!(
        report["return_code"],
        if no_check { 0 } else { -101 },
        "{root:?}"
    );
    assert!(native.stderr.is_empty(), "{root:?}");
    assert_eq!(
        native_cwd.join("out/plate_1.gcode").is_file(),
        no_check,
        "{root:?}"
    );
    if no_check {
        assert!(
            report["sliced_plates"][0]["warning_message"]
                .as_str()
                .unwrap()
                .contains("Object: other-cube.stl")
        );
    } else {
        assert!(report.get("sliced_plates").is_none());
        assert_eq!(report["warnings"], serde_json::json!([]));
    }

    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
    let ares_report = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&ares_report) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );

    let native_log = fs::read_to_string(native_cwd.join("00000.log")).unwrap();
    let ares_log = fs::read_to_string(ares_cwd.join("00000.log")).unwrap();
    assert_eq!(
        critical_records(&native_log).len(),
        if no_check { 2 } else { 0 }
    );
    assert_eq!(
        critical_records(&ares_log),
        critical_records(&native_log),
        "{root:?}"
    );
    for log in [&native_log, &ares_log] {
        assert_eq!(
            log.matches("gcode path conflicts found between ").count(),
            usize::from(!no_check),
            "{root:?}"
        );
        assert_eq!(
            log.matches("plate 1: found slicing result conflict!\n")
                .count(),
            usize::from(!no_check),
            "{root:?}"
        );
    }
}

fn critical_records(log: &str) -> Vec<&str> {
    log.split("plate 1: found slicing warnings: ")
        .skip(1)
        .map(|entry| {
            let (_, tail) = entry.split_once(", no_check=").unwrap();
            let first_line = tail.split_once('\n').unwrap().0;
            &entry[..entry.len() - tail.len() + first_line.len()]
        })
        .collect()
}
