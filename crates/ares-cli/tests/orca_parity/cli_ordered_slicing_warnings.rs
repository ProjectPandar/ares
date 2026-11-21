#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::cli_no_check::{inventory, run};
use crate::{
    artifacts, native_environment, native_wall_clock::result_comparison_bytes, runner::OrcaRunner,
};

#[test]
fn cli_checked_slice_logs_support_warning_before_rejecting_empty_layers() {
    compare(&["--slice", "1"], false);
}

#[test]
fn cli_strict_slice_stops_after_support_warning_before_empty_layers() {
    compare(&["--strict", "--slice", "1"], true);
}

fn compare(actions: &[&str], strict: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        native_environment::skipped("CLI ordered warning Native unavailable");
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-ordered-slicing-warnings-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("cantilever-with-gap.3mf");
    fs::write(
        &input,
        include_bytes!("../../../../tests/parity/ordered-slicing-warnings/cantilever-with-gap.3mf"),
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

    assert_eq!(native.status.code(), Some(156), "{root:?}");
    assert!(native.stderr.is_empty(), "{root:?}");
    assert_eq!(
        inventory(&native_cwd),
        ["00000.log", "out/", "out/result.json"],
        "{root:?}"
    );
    assert_eq!(report["return_code"], -100, "{root:?}");
    assert_eq!(report["strict_mode"], strict, "{root:?}");
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1, "{root:?}");
    assert_eq!(
        warnings[0]["class"], "slicing_warning_non_critical",
        "{root:?}"
    );
    assert_eq!(warnings[0]["plate_id"], 1, "{root:?}");
    let support = warnings[0]["text"].as_str().unwrap();
    assert!(support.contains("floating regions"), "{root:?}");
    let stopped_at = report["sliced_plates"][0]["warning_message"]
        .as_str()
        .unwrap();
    let mut expected = vec![format!(
        "plate 1: found NON_CRITICAL slicing warnings: {support}\n"
    )];
    if strict {
        assert_eq!(stopped_at, support, "{root:?}");
    } else {
        assert!(
            stopped_at.starts_with("The object has empty layers between "),
            "{root:?}"
        );
        expected.push(format!(
            "plate 1: found slicing warnings: {stopped_at}, no_check=0\n"
        ));
    }
    assert_eq!(
        warning_sequence(&native_cwd, &expected),
        (0..expected.len()).collect::<Vec<_>>(),
        "Native warning sequence: {root:?}"
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
    assert_eq!(
        warning_sequence(&ares_cwd, &expected),
        warning_sequence(&native_cwd, &expected),
        "post-slice warning sequence differs: {root:?}"
    );
}

fn warning_sequence(cwd: &Path, records: &[String]) -> Vec<usize> {
    let log = fs::read_to_string(cwd.join("00000.log")).unwrap();
    log.match_indices("plate 1: found ")
        .filter_map(|(start, prefix)| {
            let suffix = &log[start + prefix.len()..];
            if !suffix.starts_with("NON_CRITICAL slicing warnings: ")
                && !suffix.starts_with("slicing warnings: ")
            {
                return None;
            }
            Some(
                records
                    .iter()
                    .position(|record| log[start..].starts_with(record))
                    .unwrap_or_else(|| panic!("unexpected plate warning: {cwd:?}")),
            )
        })
        .collect()
}
