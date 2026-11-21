//! Public CLI parity for the BBL saved-project version admission.
#![cfg(target_os = "linux")]

use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    golden::{GeneratorKind, normalize_one_generator_line},
    native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

struct RunOutput {
    status: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    result: Vec<u8>,
    gcode: Option<Vec<u8>>,
}

#[test]
fn cli_rejects_newer_saved_project_without_allow_flag_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("cli-allow-newer-reject-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = write_project(&artifacts, newer_project());
    let native = run_native(&runner, &project, &artifacts.join("native"), false);
    let ares = run_ares(&project, &artifacts.join("ares"), false);
    assert_eq!(native.status, Some(232), "{artifacts:?}");
    assert_eq!(ares.status, native.status, "{artifacts:?}");
    assert_eq!(ares.stdout, native.stdout, "stdout: {artifacts:?}");
    assert_eq!(ares.stderr, native.stderr, "stderr: {artifacts:?}");
    assert_eq!(
        result_comparison_bytes(&ares.result),
        result_comparison_bytes(&native.result),
        "result: {artifacts:?}"
    );
    assert!(
        native.gcode.is_none(),
        "Native unexpectedly exported G-code"
    );
    assert!(ares.gcode.is_none(), "Ares unexpectedly exported G-code");
}

#[test]
fn cli_allows_newer_saved_project_with_explicit_flag_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("cli-allow-newer-accept-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = write_project(&artifacts, newer_project());
    let native = run_native(&runner, &project, &artifacts.join("native"), true);
    let ares = run_ares(&project, &artifacts.join("ares"), true);
    assert_eq!(native.status, Some(0), "{artifacts:?}");
    assert_accepted_match(&native, (&ares, GeneratorKind::Ares), &artifacts);
}

#[test]
fn cli_allow_newer_file_does_not_change_current_version_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("cli-allow-newer-control-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = write_project(&artifacts, current_project());
    let native_default = run_native(&runner, &project, &artifacts.join("native-default"), false);
    let native_allowed = run_native(&runner, &project, &artifacts.join("native-allowed"), true);
    let ares_default = run_ares(&project, &artifacts.join("ares-default"), false);
    let ares_allowed = run_ares(&project, &artifacts.join("ares-allowed"), true);
    assert_eq!(native_default.status, Some(0), "{artifacts:?}");
    assert_eq!(native_allowed.status, Some(0), "{artifacts:?}");
    assert_accepted_match(
        &native_default,
        (&ares_default, GeneratorKind::Ares),
        &artifacts,
    );
    assert_accepted_match(
        &native_allowed,
        (&ares_allowed, GeneratorKind::Ares),
        &artifacts,
    );
    assert_accepted_match(
        &native_default,
        (&native_allowed, GeneratorKind::Orca),
        &artifacts,
    );
}

fn assert_accepted_match(
    expected: &RunOutput,
    (actual, actual_kind): (&RunOutput, GeneratorKind),
    artifacts: &Path,
) {
    assert_eq!(actual.status, expected.status, "status: {artifacts:?}");
    assert_eq!(actual.stdout, expected.stdout, "stdout: {artifacts:?}");
    assert_eq!(actual.stderr, expected.stderr, "stderr: {artifacts:?}");
    assert_eq!(
        result_comparison_bytes(&actual.result),
        result_comparison_bytes(&expected.result),
        "result: {artifacts:?}"
    );
    let expected = normalize_one_generator_line(
        expected.gcode.as_deref().expect("missing Native G-code"),
        GeneratorKind::Orca,
    )
    .unwrap();
    let actual = normalize_one_generator_line(
        actual.gcode.as_deref().expect("missing G-code"),
        actual_kind,
    )
    .unwrap();
    if let Some(difference) = crate::golden::first_difference(&expected, &actual) {
        panic!("G-code: {difference}; {artifacts:?}");
    }
}

fn run_native(runner: &OrcaRunner, project: &Path, output: &Path, allow: bool) -> RunOutput {
    fs::create_dir_all(output).unwrap();
    let mut args = vec!["--slice", "0", "--outputdir"];
    let output_path = output.to_str().unwrap();
    args.push(output_path);
    if allow {
        args.push("--allow-newer-file=1");
    }
    args.push(project.to_str().unwrap());
    let process = Command::new(runner.bin()).args(args).output().unwrap();
    read_output(
        process.status.code(),
        process.stdout,
        process.stderr,
        output.join("result.json"),
        output.join("plate_1.gcode"),
    )
}

fn run_ares(project: &Path, output: &Path, allow: bool) -> RunOutput {
    fs::create_dir_all(output).unwrap();
    let gcode = output.join("plate_1.gcode");
    let mut command = crate::runner::application::cli_command();
    command.arg("slice");
    if allow {
        command.arg("--allow-newer-file=1");
    }
    let process = command.arg("-o").arg(&gcode).arg(project).output().unwrap();
    read_output(
        process.status.code(),
        process.stdout,
        process.stderr,
        output.join("result.json"),
        gcode,
    )
}

fn read_output(
    status: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    result: PathBuf,
    gcode: PathBuf,
) -> RunOutput {
    RunOutput {
        status,
        stdout,
        stderr,
        result: fs::read(&result).unwrap_or_else(|error| panic!("{result:?}: {error}")),
        gcode: fs::read(gcode).ok(),
    }
}

fn write_project(artifacts: &Path, project: Vec<u8>) -> PathBuf {
    let path = artifacts.join("input.3mf");
    fs::write(&path, project).unwrap();
    path
}

fn newer_project() -> Vec<u8> {
    with_version("3.5.0")
}

fn current_project() -> Vec<u8> {
    with_version("2.5.0")
}

fn with_version(version: &str) -> Vec<u8> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/parity/orca_cli_ender3.3mf");
    let input = fs::read(source).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut output);
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();
            if file.name() == "3D/3dmodel.model" {
                let text = String::from_utf8(bytes).unwrap();
                let marker = r#"<metadata name="OrcaSlicer">2.4.2</metadata>"#;
                let replacement = format!(r#"<metadata name="OrcaSlicer">{version}</metadata>"#);
                assert_eq!(text.matches(marker).count(), 1);
                bytes = text.replacen(marker, &replacement, 1).into_bytes();
            }
            writer
                .start_file(
                    file.name(),
                    zip::write::SimpleFileOptions::default().compression_method(file.compression()),
                )
                .unwrap();
            std::io::Write::write_all(&mut writer, &bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    output.into_inner()
}
