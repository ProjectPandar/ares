use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
    process::Command,
};

use serde_json::{Value, json};

use super::archive_members;
use crate::OrcaRunner;

const CUBE: &[u8] =
    include_bytes!("../../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");
const SETTINGS: &str = "Metadata/project_settings.config";
const DIRTY: &str = "different_settings_to_system";

#[test]
fn legacy_feature_filament_scalar_one_inherits_with_sorted_precedence() {
    check_selectors("1", "0", "feature-inherit-");
}

#[test]
fn legacy_feature_filament_explicit_scalar_survives_with_sorted_precedence() {
    check_selectors("2", "2", "feature-explicit-");
}

#[test]
fn legacy_feature_filament_bookkeeping_uses_current_names() {
    check_bookkeeping(
        "feature-bookkeeping-",
        "infill_extruder;wall_filament_id",
        json!(["sparse_infill_filament_id", "outer_wall_filament_id", ""]),
    );
}

#[test]
fn legacy_feature_filament_bookkeeping_preserves_overlapping_replacements() {
    check_bookkeeping(
        "feature-overlap-",
        "inner_wall_filament;wall_filament",
        json!([
            "inner_outer_wall_filament_id_id",
            "outer_wall_filament_id",
            ""
        ]),
    );
}

fn check_selectors(legacy: &str, expected_outer: &str, prefix: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = case_dir(&runner, prefix);
    // File order gives the two collisions the opposite winners to sorted key order.
    let input = project(&[
        ("perimeter_extruder", legacy),
        ("outer_wall_filament_id", "1"),
        ("sparse_infill_filament_id", "1"),
        ("infill_extruder", "1"),
    ]);
    let canonical = project(&[
        ("outer_wall_filament_id", expected_outer),
        ("sparse_infill_filament_id", "1"),
    ]);
    let native_bytes = export(runner.bin(), &root, "native", &input);
    let native = settings(&native_bytes);
    assert_eq!(native["outer_wall_filament_id"], expected_outer);
    assert_eq!(native["sparse_infill_filament_id"], "1");
    assert!(native.get("perimeter_extruder").is_none());
    assert!(native.get("infill_extruder").is_none());
    assert!(
        native_bytes == export(runner.bin(), &root, "canonical", &canonical),
        "Native legacy and canonical settings differ at {root:?}"
    );

    let actual = export(ares_bin(), &root, "ares", &input);
    let ares = settings(&actual);
    for key in ["outer_wall_filament_id", "sparse_infill_filament_id"] {
        assert_eq!(ares[key], native[key], "saved {key} at {root:?}");
    }
    assert!(actual == native_bytes, "saved settings differ at {root:?}");
}

pub(super) fn check_bookkeeping(prefix: &str, wire: &str, native_premise: Value) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = case_dir(&runner, prefix);
    let input = project(&[(DIRTY, wire)]);
    let native = settings(&export(runner.bin(), &root, "native", &input));
    assert_eq!(
        native[DIRTY], native_premise,
        "Native bookkeeping at {root:?}"
    );
    let ares = settings(&export(ares_bin(), &root, "ares", &input));
    assert_eq!(ares[DIRTY], native[DIRTY], "saved bookkeeping at {root:?}");
}

fn case_dir(runner: &OrcaRunner, prefix: &str) -> std::path::PathBuf {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(runner.work_dir())
        .unwrap()
        .keep()
}

fn ares_bin() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_ares"))
}

fn settings(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn export(bin: &Path, root: &Path, side: &str, bytes: &[u8]) -> Vec<u8> {
    let dir = root.join(side);
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("input.3mf"), bytes).unwrap();
    let output = Command::new(bin)
        .current_dir(&dir)
        .args([
            "--arrange",
            "0",
            "--orient",
            "0",
            "--export-3mf",
            "saved.3mf",
            "input.3mf",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout"), &output.stdout).unwrap();
    fs::write(dir.join("stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{side} export failed at {dir:?}");
    archive_members(&dir.join("saved.3mf"))
        .into_iter()
        .find_map(|(name, bytes)| (name == SETTINGS).then_some(bytes))
        .unwrap()
}

fn project(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut member = source.by_index(index).unwrap();
        output
            .start_file(member.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        if member.name() != SETTINGS {
            std::io::copy(&mut member, &mut output).unwrap();
            continue;
        }
        let mut fields: serde_json::Map<String, Value> = serde_json::from_reader(member).unwrap();
        for (key, _) in entries {
            fields.remove(*key);
        }
        let mut wire = serde_json::to_vec(&fields).unwrap();
        assert_eq!(wire.pop(), Some(b'}'));
        for (key, value) in entries {
            wire.push(b',');
            serde_json::to_writer(&mut wire, key).unwrap();
            wire.push(b':');
            serde_json::to_writer(&mut wire, value).unwrap();
        }
        wire.push(b'}');
        output.write_all(&wire).unwrap();
    }
    output.finish().unwrap().into_inner()
}
