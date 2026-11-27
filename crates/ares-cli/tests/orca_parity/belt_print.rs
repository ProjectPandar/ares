//! BeltSliceStrategy, GCode::collect_layers_to_print, BeltBrim, and BeltKinematics.

#[path = "belt_print/brim.rs"]
mod brim;
#[path = "belt_print/volumes.rs"]
mod volumes;

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

#[cfg(target_os = "linux")]
#[test]
fn rejects_a_gap_between_belt_extrusions_without_flagging_the_lead_in() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt empty-layer warnings require Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir();
    let out = work.join("out");
    fs::create_dir(&out).unwrap();
    fs::write(work.join("input.3mf"), project(true, false, &[], true)).unwrap();
    let native = stages::run(
        runner.bin(),
        ARGS.map(str::to_owned),
        stages::Stage::Slice,
        work,
    )
    .unwrap_err();
    assert_eq!(native.captured.unwrap().code, Some(156), "{work:?}");
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    assert_eq!(run_ares(work).status.code(), Some(156), "{work:?}");

    let report = |folder: &str| -> Value {
        let folder = work.join(folder);
        assert!(!folder.join("plate_1.gcode").exists(), "{folder:?}");
        let result =
            serde_json::from_slice::<Value>(&fs::read(folder.join("result.json")).unwrap())
                .unwrap();
        assert_eq!(result["return_code"], -100, "{folder:?}");
        result
    };
    let native = report("native");
    let ares = report("out");
    let expected = &native["sliced_plates"][0]["warning_message"];
    assert!(
        expected
            .as_str()
            .unwrap()
            .starts_with("The object has empty layers between 120 and 130.2 "),
        "{work:?}: {expected}"
    );
    assert_eq!(
        ares["sliced_plates"][0]["warning_message"], *expected,
        "{work:?}"
    );
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
fn rejects_belt_support_without_gcode() {
    rejects("support", &[("enable_support", json!("1"))]);
}

#[test]
fn rejects_belt_purge_without_gcode() {
    rejects("belt_purge", &[("enable_belt_purge_tower", json!("1"))]);
}

fn project(belt: bool, shift: bool, edits: &[(&str, Value)], second_island: bool) -> Vec<u8> {
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
            "3D/Objects/cube10.stl_1.model" if second_island => {
                bytes = duplicate_cube_along_belt(String::from_utf8(bytes).unwrap()).into_bytes();
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

fn duplicate_cube_along_belt(model: String) -> String {
    let section = |tag: &str| {
        model
            .split_once(&format!("<{tag}>"))
            .unwrap()
            .1
            .split_once(&format!("</{tag}>"))
            .unwrap()
            .0
    };
    let vertices = section("vertices");
    let triangles = section("triangles");
    let count = vertices.matches("<vertex ").count();
    let shifted = regex::Regex::new(r#"y="([0-9]+)""#)
        .unwrap()
        .replace_all(vertices, |cap: &regex::Captures<'_>| {
            format!("y=\"{}\"", cap[1].parse::<u32>().unwrap() + 20)
        });
    let indexed = regex::Regex::new(r#"(v[123])="([0-9]+)""#)
        .unwrap()
        .replace_all(triangles, |cap: &regex::Captures<'_>| {
            format!(
                "{}=\"{}\"",
                &cap[1],
                cap[2].parse::<usize>().unwrap() + count
            )
        });
    model
        .replace(vertices, &format!("{vertices}{shifted}"))
        .replace(triangles, &format!("{triangles}{indexed}"))
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
    compare_project(name, project(belt, shift, edits, false));
}

fn compare_project(name: &str, project: Vec<u8>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt coordinates require Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir().join(name);
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(work.join("input.3mf"), project).unwrap();
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
    fs::write(work.join("input.3mf"), project(true, false, edits, false)).unwrap();
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
