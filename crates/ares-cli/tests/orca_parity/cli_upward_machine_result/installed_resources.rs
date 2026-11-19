//! OrcaSlicer.cpp:8185–8194 locates portable resources; :2820–2825 reads the original machine there.

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use serde_json::{Value, json};

use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{self, EarlyExit, OrcaRunner},
};

const ORIGINAL_MACHINE: &str = "Bambu Lab H2D 0.4 nozzle.json";
const UPWARD: &str = "upward_compatible_machine";

#[test]
fn cli_invalid_values_reads_installed_resources() {
    installed_resources(false);
}

#[test]
fn cli_invalid_values_dump_moves_reads_installed_resources() {
    installed_resources(true);
}

fn installed_resources(dump_moves: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-installed-resources-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let repo = runner::repo_root();
    let stock = repo.join("tests/parity/cli-upward-invalid-values/firmware-wipe.3mf");
    let native = runner
        .early_cli_reference(&stock, EarlyExit::InvalidValues)
        .unwrap();
    fs::write(root.join("native.stdout"), &native.stdout).unwrap();
    fs::write(root.join("native.stderr"), &native.stderr).unwrap();
    fs::write(root.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        root.join("native.receipt.json"),
    )
    .unwrap();
    let native_result = result_comparison_bytes(&native.result_json);
    let upward = json!(["Bambu Lab H2D Pro 0.4 nozzle"]);
    assert_eq!(json_value(&native_result)[UPWARD], upward);

    let input = root.join("firmware-wipe.3mf");
    write_project_without_upward(&stock, &input, &upward);

    let prefix = root.join("install");
    let binary = prefix.join("bin/ares");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ares"), &binary).unwrap();
    let empty = root.join("empty");
    fs::create_dir(&empty).unwrap();
    let cwd = root.join("command");
    fs::create_dir(&cwd).unwrap();

    let (missing, missing_result) = slice_invalid_values(
        &binary,
        &cwd,
        &input,
        &root.join("uninstalled"),
        None,
        dump_moves,
    );
    assert_eq!(missing.stdout, native.stdout);
    assert_eq!(missing.stderr, native.stderr);
    assert!(json_value(&missing_result).get(UPWARD).is_none());

    let resources = prefix.join("resources");
    let machine_dir = resources.join("profiles/BBL/machine_full");
    fs::create_dir_all(&machine_dir).unwrap();
    // `machine_full` contains a flat resource, not the vendor's inherited fragment.
    let source = repo.join("tests/parity/machine-full-resource/stock-machine-full.json");
    let machine = fs::read(source).unwrap();
    assert_eq!(json_value(&machine)[UPWARD], upward);
    fs::write(machine_dir.join(ORIGINAL_MACHINE), machine).unwrap();

    let (fallback, fallback_result) = slice_invalid_values(
        &binary,
        &cwd,
        &input,
        &root.join("explicit-empty"),
        Some(&empty),
        dump_moves,
    );
    assert_eq!(fallback.stdout, native.stdout);
    assert_eq!(fallback.stderr, native.stderr);
    assert_eq!(fallback_result, missing_result);

    let (explicit, explicit_result) = slice_invalid_values(
        &binary,
        &cwd,
        &input,
        &root.join("explicit"),
        Some(&resources),
        dump_moves,
    );
    assert_eq!(explicit.stdout, native.stdout);
    assert_eq!(explicit.stderr, native.stderr);
    assert_eq!(explicit_result, native_result);

    let (installed, installed_result) = slice_invalid_values(
        &binary,
        &cwd,
        &input,
        &root.join("installed"),
        None,
        dump_moves,
    );
    assert_eq!(installed.stdout, native.stdout);
    assert_eq!(installed.stderr, native.stderr);
    assert_eq!(
        json_value(&installed_result)[UPWARD],
        upward,
        "the no-flag invocation must load the installed machine: {root:?}"
    );
    assert_eq!(installed_result, native_result);
}

fn slice_invalid_values(
    binary: &Path,
    cwd: &Path,
    input: &Path,
    dest: &Path,
    resources: Option<&Path>,
    dump_moves: bool,
) -> (Output, Vec<u8>) {
    fs::create_dir(dest).unwrap();
    let gcode = dest.join("plate_1.gcode");
    let moves = dest.join("moves.csv");
    let mut command = Command::new(binary);
    command.current_dir(cwd).args(["slice", "-o"]).arg(&gcode);
    if let Some(resources) = resources {
        command.arg("--resources-dir").arg(resources);
    }
    if dump_moves {
        command.arg("--dump-moves").arg(&moves);
    }
    command.arg(input);
    fs::write(dest.join("command.txt"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(dest.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(dest.join("ares.stderr"), &output.stderr).unwrap();
    fs::write(
        dest.join("ares.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(238), "{dest:?}");
    assert!(!gcode.exists() && !moves.exists(), "{dest:?}");
    let result = fs::read(dest.join("result.json")).unwrap();
    let parsed = json_value(&result);
    assert_eq!(parsed["return_code"], -18);
    assert_eq!(parsed["prepare_time"], 0);
    assert_eq!(parsed["export_time"], 0);
    (output, result)
}

fn write_project_without_upward(stock: &Path, input: &Path, upward: &Value) {
    let mut archive = zip::ZipArchive::new(fs::File::open(stock).unwrap()).unwrap();
    let mut writer = zip::ZipWriter::new(fs::File::create(input).unwrap());
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_reader(entry).unwrap();
            assert_eq!(&settings[UPWARD], upward);
            settings[UPWARD] = json!([]);
            writer
                .start_file(
                    "Metadata/project_settings.config",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(settings.to_string().as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap();
}

fn json_value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes)).unwrap()
}
