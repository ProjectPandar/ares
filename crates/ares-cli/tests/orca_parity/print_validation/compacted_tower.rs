#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::{Command, Output},
};

use serde_json::{Value, json};

const R1: &[u8] = include_bytes!("../../../../../tests/parity/semm-ramming/R1.3mf");
const DETAIL: &str = "The relative position of the model and the prime tower does not meet the requirements of the \"No sparse layers\" feature. Please adjust their relative positions, lower the model height, or turn off \"No sparse layers\".";

#[test]
fn enabled_compaction_rejects_the_original_layout() {
    compare("compacted-clearance-on", &placed_tower(None), -64, false);
}

#[test]
fn disabled_compaction_keeps_whole_gcode_parity() {
    compare("compacted-clearance-off", R1, 0, true);
}

#[test]
fn enabled_compaction_accepts_a_clear_layout() {
    compare(
        "compacted-clearance-safe",
        &placed_tower(Some("20")),
        0,
        false,
    );
}

fn placed_tower(x: Option<&str>) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(R1)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut changed = false;
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut config: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(config["wipe_tower_no_sparse_layers"], "0");
            assert_eq!(config["wipe_tower_x"], json!(["150"]));
            config["wipe_tower_no_sparse_layers"] = json!("1");
            if let Some(x) = x {
                config["wipe_tower_x"] = json!([x]);
            }
            bytes = serde_json::to_vec(&config).unwrap();
            changed = true;
        }
        output
            .start_file(
                member.name(),
                zip::write::SimpleFileOptions::default().compression_method(member.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(changed);
    output.finish().unwrap().into_inner()
}

fn compare(label: &str, input: &[u8], code: i32, strict_gcode: bool) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = root.join("input.3mf");
    fs::write(&project, input).unwrap();
    let native_dir = root.join("native");
    let ares_dir = root.join("ares");
    fs::create_dir(&native_dir).unwrap();
    fs::create_dir(&ares_dir).unwrap();
    let native = run(
        &root,
        "native",
        Command::new(runner.bin()).args(
            crate::runner::application::native_arguments(vec![
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                native_dir.display().to_string(),
                project.display().to_string(),
            ])
            .unwrap(),
        ),
    );
    let native_gcode = native_dir.join("plate_1.gcode");
    let native_report = fs::read(native_dir.join("result.json")).unwrap();
    let native_json: Value = serde_json::from_slice(&native_report).unwrap();
    assert_eq!(
        native.status.code(),
        Some(code.rem_euclid(256)),
        "Native exit: {root:?}"
    );
    assert_eq!(native_json["return_code"], code, "Native report: {root:?}");
    if code == -64 {
        assert!(
            !native_gcode.exists(),
            "Native G-code on rejection: {root:?}"
        );
        assert!(
            String::from_utf8_lossy(&native.stderr)
                .lines()
                .any(|line| line == DETAIL),
            "Native diagnostic: {root:?}"
        );
    }
    let ares_gcode = ares_dir.join("plate_1.gcode");
    let mut cli = Command::new(env!("CARGO_BIN_EXE_ares"));
    cli.args(["slice", "-o"]).arg(&ares_gcode).arg(&project);
    if let Some(supply) = crate::runner::application::project_supply().unwrap() {
        cli.arg("--project-preset-catalog")
            .arg(&supply.catalog_path);
    }
    let ares = run(&root, "ares", &mut cli);
    assert_eq!(
        ares.status.code(),
        native.status.code(),
        "CLI exit: {root:?}"
    );
    let ares_report = fs::read(ares_dir.join("result.json")).unwrap();
    if code == -64 {
        use crate::native_wall_clock::result_comparison_bytes;
        assert_eq!(
            result_comparison_bytes(&ares_report),
            result_comparison_bytes(&native_report),
            "whole result.json: {root:?}"
        );
        assert!(
            String::from_utf8_lossy(&ares.stderr)
                .lines()
                .any(|line| line == DETAIL),
            "Ares diagnostic: {root:?}"
        );
        assert!(!ares_gcode.exists(), "CLI G-code on rejection: {root:?}");
        let error =
            ares_core::validate_project_plate(input, None, &ares_core::ConfigOverrides::default())
                .unwrap_err();
        assert!(
            matches!(error, ares_core::SliceError::LayeredPrintCollision { message, plate_index: 1, .. } if message == DETAIL),
            "structured error: {root:?}"
        );
    } else {
        let ares_json: Value = serde_json::from_slice(&ares_report).unwrap();
        assert_eq!(ares_json["return_code"], 0, "Ares report: {root:?}");
        let reference = fs::read(native_gcode).unwrap();
        let actual = fs::read(ares_gcode).unwrap();
        assert!(
            !reference.is_empty() && !actual.is_empty(),
            "G-code missing: {root:?}"
        );
        if strict_gcode {
            crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
                .unwrap_or_else(|difference| panic!("{root:?}: {difference}"));
        }
    }
}

fn run(root: &Path, side: &str, command: &mut Command) -> Output {
    let argv: Vec<_> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_str().unwrap())
        .collect();
    fs::write(
        root.join(format!("{side}.argv.json")),
        serde_json::to_vec_pretty(&argv).unwrap(),
    )
    .unwrap();
    let output = command.current_dir(root).output().unwrap();
    fs::write(root.join(format!("{side}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{side}.stderr")), &output.stderr).unwrap();
    fs::write(
        root.join(format!("{side}.exit")),
        format!("{}\n", output.status),
    )
    .unwrap();
    eprintln!("{side}: {root:?}");
    output
}
