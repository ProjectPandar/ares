#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::Value;

use crate::{artifacts, native_wall_clock::result_comparison_bytes, runner::OrcaRunner};

#[test]
fn cli_strict_off_reports_warning_and_exports_gcode() {
    compare("off", &[], 0, &["plate_1.gcode", "result.json"]);
}

#[test]
fn cli_strict_refuses_noncritical_warning_before_export() {
    compare("on", &["--strict"], 156, &["result.json"]);
}

#[test]
fn cli_strict_rejects_no_check_before_loading_input() {
    compare(
        "conflict",
        &["--strict", "--no-check"],
        254,
        &["result.json"],
    );
}

fn compare(label: &str, flags: &[&str], exit: i32, expected_files: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-strict-{label}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    if exit != 254 {
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"),
            &input,
        )
        .unwrap();
    }
    let run = |bin: &Path, name: &str| {
        let cwd = root.join(name);
        let out = cwd.join("out");
        fs::create_dir_all(&out).unwrap();
        let result = Command::new(bin)
            .current_dir(&cwd)
            .args(["--slice", "1", "--outputdir", "out"])
            .args(flags)
            .arg(&input)
            .output()
            .unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
        fs::write(cwd.join("stderr"), &result.stderr).unwrap();
        fs::write(cwd.join("stdout"), &result.stdout).unwrap();
        assert_eq!(result.status.code(), Some(exit), "{cwd:?}");
        let mut files = fs::read_dir(&out)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        files.sort();
        assert_eq!(files, expected_files, "{cwd:?}");
        (result.stderr, fs::read(out.join("result.json")).unwrap())
    };
    let (native_stderr, native_report) = run(runner.bin(), "native");
    let report: Value = serde_json::from_slice(&native_report).unwrap();
    assert_eq!(report["strict_mode"], exit != 0);
    let warnings = report["warnings"].as_array().unwrap();
    if exit == 254 {
        assert_eq!(
            native_stderr,
            b"--strict cannot be combined with --no-check\n"
        );
        assert_eq!(report["return_code"], -2);
        assert_eq!(report["plate_index"], 0);
        assert!(warnings.is_empty());
        assert!(report.get("sliced_plates").is_none());
    } else {
        assert!(native_stderr.is_empty());
        assert_eq!(report["return_code"], if exit == 0 { 0 } else { -100 });
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0]["class"], "slicing_warning_non_critical");
        assert_eq!(warnings[0]["plate_id"], 1);
        assert!(
            warnings[0]["text"]
                .as_str()
                .unwrap()
                .contains("floating cantilever")
        );
        assert_eq!(
            report["sliced_plates"][0]["warning_message"],
            warnings[0]["text"]
        );
    }
    let (ares_stderr, ares_report) = run(Path::new(env!("CARGO_BIN_EXE_ares")), "ares");
    assert_eq!(ares_stderr, native_stderr, "{root:?}");
    assert_eq!(
        result_comparison_bytes(&ares_report),
        result_comparison_bytes(&native_report),
        "{root:?}"
    );
}
