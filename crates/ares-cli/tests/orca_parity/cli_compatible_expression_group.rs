use crate::runner::{
    OrcaRunner, repo_root,
    stages::{self, Stage},
};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/modifier-wall-loops-classic.3mf");
const NO_RESOLVABLE_FILAMENT_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf");

#[cfg(unix)]
#[test]
fn plain_cli_uses_installation_resources_for_filament_expression_groups() {
    run_case(PROJECT, true);
}

#[cfg(unix)]
#[test]
fn plain_cli_does_not_materialize_groups_without_a_resolvable_filament() {
    run_case(
        &with_unknown_filament_id(NO_RESOLVABLE_FILAMENT_PROJECT),
        false,
    );
}

#[cfg(unix)]
fn run_case(project: &[u8], expect_process_group: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("cli-compatible-expression-group-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    fs::write(&input, project).unwrap();

    let native_output = directory.join("native");
    fs::create_dir(&native_output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            native_output.to_string_lossy().into_owned(),
            input.to_string_lossy().into_owned(),
        ],
        Stage::Slice,
        &directory,
    )
    .unwrap_or_else(|error| panic!("Native slice: {error}; {directory:?}"));

    let installation = directory.join("installation");
    let bin_dir = installation.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let bin = bin_dir.join("ares");
    fs::copy(env!("CARGO_BIN_EXE_ares"), &bin).unwrap();
    std::os::unix::fs::symlink(
        repo_root().join("OrcaSlicer/resources"),
        installation.join("resources"),
    )
    .unwrap();
    let ares_output = directory.join("ares");
    fs::create_dir(&ares_output).unwrap();
    let status = Command::new(&bin)
        .args(["--slice", "1", "--outputdir"])
        .arg(&ares_output)
        .arg(&input)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "Ares slice failed: {status}; {directory:?}"
    );

    let native = fs::read(native_output.join("plate_1.gcode")).unwrap();
    let actual = fs::read(ares_output.join("plate_1.gcode")).unwrap();
    let expected_lines = expression_group_lines(&native);
    let actual_lines = expression_group_lines(&actual);
    assert_eq!(
        expected_lines
            .iter()
            .any(|line| line.starts_with("; compatible_process_expression_group = ")),
        expect_process_group,
        "Native trigger witness; {directory:?}"
    );
    assert!(
        !expected_lines
            .iter()
            .any(|line| line.starts_with("; compatible_machine_expression_group = ")),
        "machine-group behavior is not part of this witness; {directory:?}"
    );
    assert_eq!(actual_lines, expected_lines, "see {directory:?}");
}

fn with_unknown_filament_id(source: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["filament_settings_id"] = serde_json::json!(["unresolvable filament"]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn expression_group_lines(bytes: &[u8]) -> Vec<String> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            line.starts_with("; compatible_machine_expression_group = ")
                || line.starts_with("; compatible_process_expression_group = ")
        })
        .map(str::to_owned)
        .collect()
}
