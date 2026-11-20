#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use crate::{artifacts, native_wall_clock::result_comparison_bytes, runner::OrcaRunner};

#[test]
fn cli_export_settings_stl_file_without_slicing() {
    compare(
        &["--export-settings", "out/settings.json"],
        0,
        &["out/settings.json", "result.json"],
    );
}

#[test]
fn cli_export_settings_stl_file_before_slice() {
    compare(
        &[
            "--export-settings",
            "out/settings.json",
            "--slice",
            "1",
            "--outputdir",
            "out",
        ],
        0,
        &["out/plate_1.gcode", "out/result.json", "out/settings.json"],
    );
}

#[test]
fn cli_export_settings_stl_file_after_slice() {
    compare(
        &[
            "--slice",
            "1",
            "--outputdir",
            "out",
            "--export-settings",
            "out/settings.json",
        ],
        0,
        &["out/plate_1.gcode", "out/result.json", "out/settings.json"],
    );
}

#[test]
fn cli_export_settings_stl_file_survives_a_later_slice_error() {
    compare(
        &[
            "--export-settings",
            "out/settings.json",
            "--mtcpp",
            "11",
            "--slice",
            "1",
            "--outputdir",
            "out",
        ],
        197,
        &["out/result.json", "out/settings.json"],
    );
}

#[test]
fn cli_export_settings_stl_file_not_written_after_a_slice_error() {
    compare(
        &[
            "--mtcpp",
            "11",
            "--slice",
            "1",
            "--outputdir",
            "out",
            "--export-settings",
            "out/settings.json",
        ],
        197,
        &["out/result.json"],
    );
}

#[test]
fn cli_export_settings_stl_stdout_is_only_the_json_document() {
    compare(&["--export-settings", "-"], 0, &["result.json"]);
}

#[test]
fn cli_export_settings_stl_stdout_conflicts_with_slice() {
    compare(
        &[
            "--export-settings",
            "-",
            "--slice",
            "1",
            "--outputdir",
            "out",
        ],
        254,
        &["out/result.json"],
    );
}

#[test]
fn cli_export_settings_stl_stdout_conflicts_with_arrange() {
    compare(
        &["--export-settings", "-", "--arrange", "0"],
        254,
        &["result.json"],
    );
}

fn compare(args: &[&str], exit: i32, inventory: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-export-settings-stl-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    super::cli_mtcpp::write_fixture(&root, true);
    let native = run(runner.bin(), &root, "native", args);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", args);

    assert_eq!(native.exit, Some(exit), "{root:?}");
    assert_eq!(native.inventory, inventory, "{root:?}");
    let report: Value = serde_json::from_slice(&native.report).unwrap();
    assert_eq!(
        report["return_code"],
        if exit == 0 { 0 } else { exit - 256 }
    );
    if let Some(settings) = &native.settings {
        let settings: Value = serde_json::from_slice(settings).unwrap();
        assert_eq!(settings["printer_settings_id"], "Synthetic single nozzle");
        assert_eq!(settings["print_settings_id"], "Synthetic walls");
        assert_eq!(settings["filament_settings_id"], json!(["Synthetic PLA"]));
        assert_eq!(settings["layer_height"], "0.2");
    }

    assert_eq!(ares.exit, native.exit, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.inventory, native.inventory, "{root:?}");
    assert!(
        ares.report == native.report,
        "result.json differs: {root:?}"
    );
    assert!(
        ares.settings == native.settings,
        "settings bytes differ: {root:?}"
    );
    assert!(ares.stdout == native.stdout, "stdout differs: {root:?}");
}

struct Artifacts {
    exit: Option<i32>,
    stderr: Vec<u8>,
    stdout: Vec<u8>,
    inventory: Vec<String>,
    report: Vec<u8>,
    settings: Option<Vec<u8>>,
}

fn run(bin: &Path, root: &Path, engine: &str, args: &[&str]) -> Artifacts {
    let work = root.join(engine);
    let cwd = work.join("cwd");
    let out = cwd.join("out");
    let tmp = work.join("tmp");
    fs::create_dir_all(&out).unwrap();
    fs::create_dir(&tmp).unwrap();
    let mut command = Command::new(bin);
    if engine == "native" {
        // Keep the Native diagnostic sink outside the directory under inventory.
        command.arg("--logfile").arg(work.join("native.log"));
    }
    let result = command
        .current_dir(&cwd)
        .env("TMPDIR", tmp)
        .env("ORCA_250_JOB_APPDIR", work.join("app"))
        .env("ORCA_JOB_APPDIR", work.join("app"))
        .args(args)
        .arg("--load-settings")
        .arg(format!(
            "{};{}",
            root.join("machine.json").display(),
            root.join("process.json").display()
        ))
        .arg("--load-filaments")
        .arg(root.join("filament.json"))
        .arg(root.join("first.stl"))
        .output()
        .unwrap();
    let exit = result.status.code();
    fs::write(work.join("exit"), format!("{exit:?}\n")).unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    let mut inventory = fs::read_dir(&cwd)
        .unwrap()
        .chain(fs::read_dir(&out).unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path != &out)
        .map(|path| {
            path.strip_prefix(&cwd)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    inventory.sort_unstable();
    let report = fs::read(out.join("result.json"))
        .or_else(|_| fs::read(cwd.join("result.json")))
        .map(|bytes| result_comparison_bytes(&bytes))
        .unwrap_or_default();
    let settings = if args.contains(&"-") && exit == Some(0) {
        Some(result.stdout.clone())
    } else {
        fs::read(out.join("settings.json")).ok()
    };
    Artifacts {
        exit,
        stderr: result.stderr,
        stdout: result.stdout,
        inventory,
        report,
        settings,
    }
}
