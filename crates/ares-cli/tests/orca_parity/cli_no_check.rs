#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path, process::Command};

use serde_json::Value;

use crate::{
    artifacts, load_settings_cli::preset_files, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_no_check_absent_reports_generic_validation_error() {
    compare("validate-default", &["--slice", "1"], true, false);
}

#[test]
fn cli_no_check_before_slice_reports_detail_but_still_rejects() {
    compare(
        "validate-before",
        &["--no-check", "--slice", "1"],
        true,
        true,
    );
}

#[test]
fn cli_no_check_after_slice_leaves_validation_report_unchanged() {
    compare(
        "validate-after",
        &["--slice", "1", "--no-check"],
        true,
        false,
    );
}

#[test]
fn cli_no_check_absent_exports_valid_cube() {
    compare("cube-default", &["--slice", "1"], false, false);
}

#[test]
fn cli_no_check_before_slice_exports_valid_cube() {
    compare("cube-before", &["--no-check", "--slice", "1"], false, false);
}

fn compare(name: &str, actions: &[&str], invalid: bool, detailed: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-no-check-{name}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let mut argv: Vec<OsString> = ["--debug", "0", "--outputdir", "out"]
        .into_iter()
        .chain(actions.iter().copied())
        .map(OsString::from)
        .collect();
    if invalid {
        let input = root.join("two-object-by-layer.3mf");
        fs::write(
            &input,
            include_bytes!(
                "../../../../tests/parity/spiral-vase-admission/two-object-by-layer.3mf"
            ),
        )
        .unwrap();
        argv.push(input.into());
    } else {
        let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
        let input = root.join("block-0.stl");
        fs::write(&input, super::direct_stl_slice::block_stl(108.0)).unwrap();
        argv.extend([
            "--load-settings".into(),
            presets[..2].join(";").into(),
            "--load-filaments".into(),
            presets[2..].join(";").into(),
            input.into(),
        ]);
    }
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    assert_eq!(
        native.status.code(),
        Some(if invalid { 205 } else { 0 }),
        "{root:?}"
    );
    let expected: &[&str] = if invalid {
        &["out/", "out/result.json"]
    } else {
        &["out/", "out/plate_1.gcode", "out/result.json"]
    };
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    assert_eq!(inventory(&native_cwd), expected, "{root:?}");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let report: Value = serde_json::from_slice(&native_report).unwrap();
    assert_eq!(report["return_code"], if invalid { -51 } else { 0 });
    if invalid {
        let stderr = std::str::from_utf8(&native.stderr).unwrap();
        assert!(stderr.contains("spiral vase mode"), "{root:?}");
        if detailed {
            assert_eq!(report["error_string"], stderr.strip_suffix('\n').unwrap());
        } else {
            assert_eq!(
                report["error_string"],
                "There are some incorrect slicing parameters in the 3mf. Please verify the slicing of all plates in Orca Slicer before uploading."
            );
        }
    }
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), expected, "{root:?}");
    let ares_report = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&ares_report) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );
    if !invalid {
        assert!(
            gcode(&ares_cwd) == gcode(&native_cwd),
            "complete G-code differs: {root:?}"
        );
    }
}

pub(super) fn run(
    bin: &Path,
    root: &Path,
    engine: &str,
    argv: &[OsString],
) -> std::process::Output {
    let run = root.join(engine);
    let cwd = root.join("cwd");
    let tmp = run.join("tmp");
    fs::create_dir_all(cwd.join("out")).unwrap();
    fs::create_dir_all(&tmp).unwrap();
    let output = Command::new(bin)
        .current_dir(&cwd)
        .env("TMPDIR", tmp)
        .env("ORCA_250_JOB_APPDIR", run.join("app"))
        .env("ORCA_JOB_APPDIR", run.join("app"))
        .args(argv)
        .output()
        .unwrap();
    fs::rename(cwd, run.join("cwd")).unwrap();
    fs::write(run.join("exit"), format!("{:?}\n", output.status.code())).unwrap();
    fs::write(run.join("stderr"), &output.stderr).unwrap();
    fs::write(run.join("stdout"), &output.stdout).unwrap();
    output
}

pub(super) fn inventory(cwd: &Path) -> Vec<String> {
    fn visit(cwd: &Path, dir: &Path, paths: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(cwd).unwrap().to_str().unwrap();
            if entry.file_type().unwrap().is_dir() {
                paths.push(format!("{relative}/"));
                visit(cwd, &path, paths);
            } else {
                paths.push(relative.to_owned());
            }
        }
    }
    let mut paths = Vec::new();
    visit(cwd, cwd, &mut paths);
    paths.sort_unstable();
    paths
}

pub(super) fn gcode(cwd: &Path) -> Vec<u8> {
    fs::read(cwd.join("out/plate_1.gcode"))
        .unwrap()
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| !line.starts_with(b"; generated by "))
        .flatten()
        .copied()
        .collect()
}
