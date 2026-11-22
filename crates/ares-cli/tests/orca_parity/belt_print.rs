//! BeltSliceStrategy, GCode::collect_layers_to_print, and BeltKinematics.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::{Command, Output},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");
const ARGS: [&str; 5] = ["--slice", "1", "--outputdir", "out", "input.3mf"];

#[test]
fn slices_in_the_tilted_frame_and_writes_machine_coordinates() {
    compare("tilted", true, false, &[]);
}

#[test]
fn preserves_belt_placement_after_translating_the_instance() {
    compare("translated", true, true, &[]);
}

#[test]
fn disabling_belt_mode_preserves_cartesian_output() {
    compare("cartesian", false, false, &[]);
}

#[test]
fn emits_belt_first_layer_bands_with_no_slowdown_ramp() {
    compare(
        "first-layer-bands",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("slow_down_layers", json!("0")),
        ],
    );
}

#[test]
fn rejects_belt_brim_without_gcode() {
    rejects("brim", &[("brim_type", json!("auto_brim"))]);
}

#[test]
fn rejects_belt_support_without_gcode() {
    rejects("support", &[("enable_support", json!("1"))]);
}

#[test]
fn rejects_belt_purge_without_gcode() {
    rejects("belt_purge", &[("enable_belt_purge_tower", json!("1"))]);
}

fn project(belt: bool, shift: bool, edits: &[(&str, Value)]) -> Vec<u8> {
    let mut reader = ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..reader.len() {
        let mut member = reader.by_index(i).unwrap();
        let name = member.name().to_owned();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match name.as_str() {
            "Metadata/project_settings.config" => {
                let mut config: serde_json::Map<String, Value> =
                    serde_json::from_slice(&bytes).unwrap();
                for (key, value) in [
                    ("belt_printer", if belt { "1" } else { "0" }),
                    ("belt_slice_rotation", "x"),
                    ("belt_slice_rotation_angle", "90"),
                    ("gcode_remap_x", "rev_x"),
                    ("gcode_remap_y", "pos_z"),
                    ("gcode_remap_z", "pos_y"),
                    ("enable_arc_fitting", "disabled"),
                ] {
                    config.insert(key.to_owned(), json!(value));
                }
                config.extend(
                    edits
                        .iter()
                        .map(|(key, value)| ((*key).to_owned(), value.clone())),
                );
                bytes = serde_json::to_vec(&config).unwrap();
            }
            "3D/3dmodel.model" if shift => {
                let model = String::from_utf8(bytes).unwrap();
                assert_eq!(model.matches("110 110 0\" printable").count(), 1);
                bytes = model
                    .replace("110 110 0\" printable", "110 130 0\" printable")
                    .into_bytes();
            }
            _ => {}
        }
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn run_ares(work: &Path) -> Output {
    let result = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(work)
        .args(ARGS)
        .output()
        .unwrap();
    fs::write(work.join("ares.stdout"), &result.stdout).unwrap();
    fs::write(work.join("ares.stderr"), &result.stderr).unwrap();
    result
}

fn compare(name: &str, belt: bool, shift: bool, edits: &[(&str, Value)]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt coordinates require Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir().join(name);
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(work.join("input.3mf"), project(belt, shift, edits)).unwrap();
    stages::run(
        runner.bin(),
        ARGS.map(str::to_owned),
        stages::Stage::Slice,
        &work,
    )
    .unwrap();
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    let result = run_ares(&work);
    assert!(
        result.status.success(),
        "Ares exited {} in {work:?}",
        result.status
    );
    let native = fs::read(work.join("native/plate_1.gcode")).unwrap();
    let actual = fs::read(out.join("plate_1.gcode")).unwrap();
    golden::compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{error}; {work:?}"));
}

fn rejects(reason: &str, edits: &[(&str, Value)]) {
    let work = tempfile::Builder::new()
        .prefix("unsupported-belt-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let out = work.join("out");
    fs::create_dir(&out).unwrap();
    fs::write(work.join("input.3mf"), project(true, false, edits)).unwrap();
    let result = run_ares(&work);
    assert!(
        !result.status.success(),
        "unsupported belt mode was accepted in {work:?}"
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains(&format!("belt_printer: {reason}")),
        "wrong rejection reason in {work:?}"
    );
    assert!(
        !out.join("plate_1.gcode").exists(),
        "rejected belt mode emitted G-code in {work:?}"
    );
}
