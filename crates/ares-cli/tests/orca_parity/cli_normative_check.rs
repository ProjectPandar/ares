#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
    process::Command,
};

use crate::{
    golden::compare_ordered_bytes_generator_only,
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, repo_root},
};

const INPUT: &str = "project.3mf";
const GCODE: &str = "plate_1.gcode";
const SETTINGS: &str = "Metadata/project_settings.config";
const CONTROL: &[u8] = include_bytes!(
    "../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
);

#[test]
fn embedded_post_process_is_rejected_only_when_normative_check_is_enabled() {
    compare(
        &[""],
        [
            (None, 237),
            (Some("--normative-check=1"), 237),
            (Some("--normative-check=0"), 0),
        ],
    );
}

#[test]
fn project_without_post_process_is_admitted_in_both_normative_modes() {
    compare(&[], [(None, 0), (Some("--normative-check=0"), 0)]);
}

#[test]
fn installed_refresh_admits_post_process_from_retained_stock_process() {
    compare_installed_refresh(&[], 0);
}

#[test]
fn installed_refresh_rejects_post_process_embedded_in_project() {
    compare_installed_refresh(&[""], 237);
}

fn compare_installed_refresh(post_process: &[&str], expected: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = super::cli_uptodate_refresh::artifact_dir();
    super::cli_uptodate_refresh::project(&root);
    let input = root.join(INPUT);
    let fixture = fs::read(root.join("retained.3mf")).unwrap();
    fs::write(&input, project(&fixture, post_process)).unwrap();

    let installation = root.join("installation");
    let bin = installation.join("bin/ares");
    fs::create_dir_all(bin.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ares"), &bin).unwrap();
    std::os::unix::fs::symlink(
        repo_root().join("OrcaSlicer/resources"),
        installation.join("resources"),
    )
    .unwrap();

    let native_work = root.join("native");
    let ares_work = root.join("ares");
    create_work(&native_work);
    create_work(&ares_work);
    let mut native = Command::new(runner.bin());
    native
        .args(["--slice", "1", "--outputdir", ".", "--uptodate"])
        .env("ORCA_250_JOB_APPDIR", native_work.join("app"))
        .env("ORCA_JOB_APPDIR", native_work.join("app"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY");
    let mut ares = Command::new(bin);
    ares.args(["slice", "--uptodate", "-o", GCODE]);
    let native = run(native, &native_work, &input, None, (expected, true));
    let actual = run(ares, &ares_work, &input, None, (expected, false));
    if expected == 0 {
        let native = fs::read(native_work.join("out").join(GCODE)).unwrap();
        let actual = fs::read(ares_work.join("out").join(GCODE)).unwrap();
        assert!(
            std::str::from_utf8(&native)
                .unwrap()
                .lines()
                .any(|line| line == "; post_process = \"\""),
            "Native did not use the retained stock option; {root:?}"
        );
        compare_ordered_bytes_generator_only(&native, &actual)
            .unwrap_or_else(|error| panic!("{root:?}: {error}"));
    } else {
        assert_eq!(actual, native, "stderr or result.json: {root:?}");
        assert_eq!(
            fs::read(ares_work.join("stdout")).unwrap(),
            fs::read(native_work.join("stdout")).unwrap(),
            "stdout: {root:?}"
        );
    }
}

fn compare(post_process: &[&str], cases: impl IntoIterator<Item = (Option<&'static str>, i32)>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-normative-check-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join(INPUT);
    fs::write(&input, project(CONTROL, post_process)).unwrap();
    let mut native = Vec::new();
    for (index, (mode, expected)) in cases.into_iter().enumerate() {
        let work = root.join(format!("native-{index}"));
        create_work(&work);
        let mut command = Command::new(runner.bin());
        command
            .args(["--slice", "0", "--outputdir", "."])
            .env("ORCA_250_JOB_APPDIR", work.join("app"))
            .env("ORCA_JOB_APPDIR", work.join("app"))
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY");
        let observed = run(command, &work, &input, mode, (expected, true));
        native.push((mode, expected, observed));
    }
    for (index, (mode, expected, reference)) in native.into_iter().enumerate() {
        let work = root.join(format!("ares-{index}"));
        create_work(&work);
        let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
        command.args(["slice", "-o", GCODE]);
        let actual = run(command, &work, &input, mode, (expected, false));
        assert_eq!(actual.0, reference.0, "stderr: {work:?}");
        assert_eq!(actual.1, reference.1, "result.json: {work:?}");
    }
}

fn create_work(work: &Path) {
    fs::create_dir_all(work.join("out")).unwrap();
    fs::create_dir(work.join("app")).unwrap();
}

fn run(
    mut command: Command,
    work: &Path,
    input: &Path,
    mode: Option<&str>,
    (expected, native): (i32, bool),
) -> (Vec<u8>, String) {
    let out = work.join("out");
    let output = command
        .current_dir(&out)
        .args(mode)
        .arg(input)
        .output()
        .unwrap();
    for (name, bytes) in [("stdout", &output.stdout), ("stderr", &output.stderr)] {
        fs::write(work.join(name), bytes).unwrap();
    }
    fs::write(work.join("exit"), format!("{:?}\n", output.status.code())).unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{mode:?}: {work:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut names: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names.sort();
    let mut expected_files = Vec::new();
    if native {
        // OrcaSlicer.cpp:1532–1537 opens its default log in the working directory.
        expected_files.push("00000.log");
    }
    if expected == 0 {
        expected_files.push(GCODE);
    }
    expected_files.push("result.json");
    assert_eq!(names, expected_files, "{work:?}");
    let report = fs::read(out.join("result.json")).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&report).unwrap();
    if expected == 237 {
        assert_eq!(document["return_code"], -19);
        assert_eq!(
            document["error_string"],
            "post_process is not supported under CLI."
        );
    } else {
        assert_eq!(document["return_code"], 0);
    }
    (
        output.stderr,
        String::from_utf8(result_comparison_bytes(&report)).unwrap(),
    )
}

fn project(input: &[u8], post_process: &[&str]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut config: serde_json::Value =
        serde_json::from_reader(archive.by_name(SETTINGS).unwrap()).unwrap();
    config["post_process"] = serde_json::json!(post_process);
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let member = archive.by_index(index).unwrap();
        if member.name() == SETTINGS {
            writer
                .start_file(SETTINGS, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer
                .write_all(&serde_json::to_vec(&config).unwrap())
                .unwrap();
        } else {
            writer.raw_copy_file(member).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
