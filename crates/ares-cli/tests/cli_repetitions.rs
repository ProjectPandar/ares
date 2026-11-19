#![cfg(target_os = "linux")]

#[path = "stock_project_whole_file/golden.rs"]
mod golden;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "orca_parity/native_wall_clock.rs"]
mod native_wall_clock;

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::{Command, Output},
};

const INPUT: &[u8] = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");

#[test]
fn repetitions_pack_two_copies_on_the_selected_plate() {
    compare("1", "2", 0, false);
}

#[test]
fn nonpositive_repetitions_leave_the_selected_plate_unchanged() {
    compare("1", "-1", 0, false);
}

#[test]
fn repetitions_reject_slice_all_without_writing_gcode() {
    compare("0", "2", 254, false);
}

#[test]
fn repetitions_restore_original_placement_when_no_copy_fits() {
    compare("1", "2", 0, true);
}

fn compare(plate: &str, count: &str, exit: i32, small_bed: bool) {
    let Some(native_bin) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let artifacts = native_environment::value("ARES_PARITY_ARTIFACT_ROOT").unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let root = tempfile::Builder::new()
        .prefix("cli-repetitions-")
        .tempdir_in(artifacts)
        .unwrap()
        .keep();
    let input = root.join("project.3mf");
    if small_bed {
        project_on_small_bed(&input);
    } else {
        fs::write(&input, INPUT).unwrap();
    }
    let native_dir = root.join("native");
    let ares_dir = root.join("ares");
    fs::create_dir(&native_dir).unwrap();
    fs::create_dir(&ares_dir).unwrap();
    let native = Command::new(native_bin)
        .current_dir(&native_dir)
        .env("ORCA_250_JOB_APPDIR", native_dir.join("app"))
        .env("ORCA_JOB_APPDIR", native_dir.join("app"))
        .arg("--datadir")
        .arg(native_dir.join("data"))
        .args(["--repetitions", count, "--slice", plate, "--outputdir", "."])
        .arg(&input)
        .output()
        .unwrap();
    record(&native_dir, &native);
    assert_eq!(native.status.code(), Some(exit), "Native: {root:?}");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command.current_dir(&ares_dir).args([
        "slice",
        "--plate",
        plate,
        "--repetitions",
        count,
        "-o",
        "plate_1.gcode",
    ]);
    if let Some(path) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") {
        command.arg("--project-preset-catalog").arg(path);
    }
    let actual = command.arg(&input).output().unwrap();
    record(&ares_dir, &actual);
    assert_eq!(
        actual.status.code(),
        Some(exit),
        "Ares: {root:?}: {}",
        String::from_utf8_lossy(&actual.stderr)
    );
    let report = |dir: &Path| {
        native_wall_clock::result_comparison_bytes(&fs::read(dir.join("result.json")).unwrap())
    };
    assert_eq!(
        report(&native_dir),
        report(&ares_dir),
        "result.json: {root:?}"
    );
    let gcode = |dir: &Path| fs::read(dir.join("plate_1.gcode"));
    if exit == 0 {
        golden::compare_ordered_bytes_generator_only(
            &gcode(&native_dir).unwrap(),
            &gcode(&ares_dir).unwrap(),
        )
        .unwrap_or_else(|error| panic!("{root:?}: {error}"));
    } else {
        for directory in [&native_dir, &ares_dir] {
            assert_eq!(
                gcode(directory).unwrap_err().kind(),
                std::io::ErrorKind::NotFound
            );
        }
    }
}

fn record(dir: &Path, output: &Output) {
    fs::write(dir.join("stdout"), &output.stdout).unwrap();
    fs::write(dir.join("stderr"), &output.stderr).unwrap();
    fs::write(dir.join("exit"), output.status.to_string()).unwrap();
}

fn project_on_small_bed(path: &Path) {
    let mut original = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut project = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for index in 0..original.len() {
        let mut member = original.by_index(index).unwrap();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match member.name() {
            "Metadata/project_settings.config" => {
                let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                settings["printable_area"] = serde_json::json!(["0x0", "18x0", "18x18", "0x18"]);
                bytes = serde_json::to_vec(&settings).unwrap();
            }
            "3D/3dmodel.model" => {
                let xml = String::from_utf8(bytes).unwrap();
                bytes = xml.replace(" 110 110 5\"", " 9 9 5\"").into_bytes();
            }
            _ => {}
        }
        project
            .start_file(member.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        project.write_all(&bytes).unwrap();
    }
    project.finish().unwrap();
}
