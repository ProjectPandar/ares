use std::{fs, path::Path, process::Command};

use serde_json::Value;

use super::{
    direct_stl_slice::square_stl, export_3mf::archive_members,
    export_3mf_sliced::compare_sliced_archives,
};
use crate::OrcaRunner;

const ARCHIVE: &str = "out.3mf";
const MACHINE: &str = r#"{"name":"Synthetic machine","type":"machine","from":"system","instantiation":"true","before_layer_change_gcode":"G92 E0"}"#;

#[test]
fn cli_top_level_stl_exports_unsliced_with_default_settings() {
    compare_unsliced(&[], 10.0, |_| {});
}

#[test]
fn cli_top_level_stl_exports_unsliced_without_arranging() {
    compare_unsliced(&["--arrange", "0"], 10.0, |members| {
        assert!(
            text_member(members, "3D/3dmodel.model")
                .contains(r#"transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1""#)
        );
    });
}

#[test]
fn cli_top_level_stl_exports_unsliced_with_forced_arrangement() {
    compare_unsliced(&["--arrange", "1"], 10.0, |members| {
        assert!(
            text_member(members, "3D/3dmodel.model")
                .contains(r#"transform="1 0 0 0 1 0 0 0 1 95 95 0" printable="1""#)
        );
    });
}

#[test]
fn cli_top_level_stl_exports_unsliced_with_partial_settings() {
    compare_unsliced(&["--load-settings", "machine.json"], 10.0, |members| {
        let config: Value =
            serde_json::from_str(text_member(members, "Metadata/project_settings.config")).unwrap();
        assert_eq!(config["before_layer_change_gcode"], "G92 E0");
    });
}

#[test]
fn cli_top_level_stl_converts_units_before_arranging() {
    for (edge, source_unit) in [
        (1.0, Some("source_in_inches")),
        (0.01, Some("source_in_meters")),
        (10.0, None),
    ] {
        compare_unsliced(&["--convert-unit", "--arrange", "1"], edge, |members| {
            let settings = text_member(members, "Metadata/model_settings.config");
            for unit in ["source_in_inches", "source_in_meters"] {
                assert_eq!(settings.contains(unit), source_unit == Some(unit));
            }
        });
    }
}

#[test]
fn cli_top_level_stl_exports_all_sliced_plates() {
    compare_sliced(&["--slice", "0"], 10.0);
}

#[test]
fn cli_top_level_stl_exports_selected_sliced_plate() {
    compare_sliced(&["--slice", "1"], 10.0);
}

#[test]
fn cli_top_level_stl_slices_and_exports_the_converted_model() {
    compare_sliced(&["--convert-unit", "--arrange", "1", "--slice", "0"], 0.01);
}

type Members = [(String, Vec<u8>)];

fn compare_unsliced(flags: &[&str], edge: f32, witness: impl FnOnce(&Members)) {
    let Some((runner, native, ares)) = prepare_pair(edge) else {
        return;
    };
    let argv = [flags, &["--export-3mf", ARCHIVE, "model.stl"]].concat();
    run_export(runner.bin(), &native, &argv);
    let expected = archive_members(&native.join(ARCHIVE));
    witness(&expected);
    assert!(!expected.iter().any(|(name, _)| name.ends_with(".gcode")
        || name.ends_with(".gcode.md5")
        || name == "Metadata/plate_1.json"));
    run_export(Path::new(env!("CARGO_BIN_EXE_ares")), &ares, &argv);
    let actual = archive_members(&ares.join(ARCHIVE));
    assert!(
        expected
            .iter()
            .map(|(name, _)| name)
            .eq(actual.iter().map(|(name, _)| name))
    );
    for ((name, expected), (_, actual)) in expected.iter().zip(&actual) {
        let mismatch = expected
            .iter()
            .zip(actual)
            .position(|(left, right)| left != right);
        assert!(
            mismatch.is_none() && expected.len() == actual.len(),
            "{name} differs at byte {mismatch:?} ({} != {} bytes) in {}",
            expected.len(),
            actual.len(),
            runner.work_dir().display()
        );
    }
}

fn compare_sliced(flags: &[&str], edge: f32) {
    let Some((runner, native, ares)) = prepare_pair(edge) else {
        return;
    };
    let argv = [
        &["--load-settings", "machine.json"][..],
        flags,
        &["--export-3mf", ARCHIVE, "model.stl"],
    ]
    .concat();
    run_export(runner.bin(), &native, &argv);
    run_export(Path::new(env!("CARGO_BIN_EXE_ares")), &ares, &argv);
    compare_sliced_archives(&native.join(ARCHIVE), &ares.join(ARCHIVE));
    for dir in [&native, &ares] {
        assert!(!dir.join("plate_1.gcode").exists());
    }
}

fn prepare_pair(edge: f32) -> Option<(OrcaRunner, std::path::PathBuf, std::path::PathBuf)> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let native = runner.work_dir().join("native");
    let ares = runner.work_dir().join("ares");
    let stl = square_stl([0.0, 0.0], edge, edge);
    for dir in [&native, &ares] {
        fs::create_dir(dir).unwrap();
        fs::write(dir.join("machine.json"), MACHINE).unwrap();
        fs::write(dir.join("model.stl"), &stl).unwrap();
    }
    Some((runner, native, ares))
}

fn run_export(bin: &Path, dir: &Path, argv: &[&str]) {
    let output = Command::new(bin)
        .current_dir(dir)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args(argv)
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{} {argv:?}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn text_member<'a>(members: &'a Members, path: &str) -> &'a str {
    let (_, bytes) = members.iter().find(|(name, _)| name == path).unwrap();
    std::str::from_utf8(bytes).unwrap()
}
