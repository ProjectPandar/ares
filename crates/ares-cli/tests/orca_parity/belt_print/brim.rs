//! Whole-G-code parity for `BeltFloorContext` and `BeltBrim` (`BeltBrim.cpp`, `GCode.cpp`).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const TWO_HOLES: &[u8] =
    include_bytes!("../../../../../tests/parity/inner-only-brim/two-holes.3mf");

#[test]
fn belt_leading_edge_brim_matches_native() {
    super::compare(
        "brim-leading-45",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("leading_edge_only")),
            ("brim_width", json!("5")),
        ],
    );
    super::compare(
        "brim-leading-45-gap",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("leading_edge_only")),
            ("brim_width", json!("5")),
            ("brim_object_gap", json!("0.2")),
        ],
    );
    super::compare(
        "brim-leading-45-extras",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("leading_edge_only")),
            ("brim_width", json!("5")),
            ("leading_brim_length", json!("8")),
            ("extra_brim_width", json!("3")),
        ],
    );
    super::compare(
        "brim-leading-45-slowdown0",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("leading_edge_only")),
            ("brim_width", json!("5")),
            ("slow_down_layers", json!("0")),
        ],
    );
}

#[test]
fn belt_outer_brim_axes_and_offsets_match_native() {
    super::compare(
        "brim-outer-45-x",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
        ],
    );
    super::compare(
        "brim-outer-45-y",
        true,
        false,
        &[
            ("belt_slice_rotation", json!("y")),
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
        ],
    );
    super::compare(
        "brim-outer-neg45-x",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("-45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
        ],
    );
    super::compare(
        "brim-outer-45-floor-offset",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
            ("belt_support_floor_offset", json!("0.5")),
        ],
    );
    super::compare(
        "brim-outer-45-zero-width-lead",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("0")),
            ("leading_brim_length", json!("5")),
        ],
    );
}

#[test]
fn belt_brim_types_and_no_brim_match_native() {
    for (name, brim_type) in [
        ("brim-type-no-brim", "no_brim"),
        ("brim-type-inner-only", "inner_only"),
        ("brim-type-outer-and-inner", "outer_and_inner"),
        ("brim-type-auto", "auto_brim"),
        ("brim-type-ears", "brim_ears"),
    ] {
        super::compare(
            name,
            true,
            false,
            &[
                ("belt_slice_rotation_angle", json!("45")),
                ("brim_type", json!(brim_type)),
                ("brim_width", json!("5")),
            ],
        );
    }
}

#[test]
fn belt_brim_tilt_window_boundaries_match_native() {
    for (name, angle) in [
        ("brim-tilt-0p5", "0.5"),
        ("brim-tilt-1", "1"),
        ("brim-tilt-85", "85"),
        ("brim-tilt-86", "86"),
    ] {
        super::compare(
            name,
            true,
            false,
            &[
                ("belt_slice_rotation_angle", json!(angle)),
                ("brim_type", json!("outer_only")),
                ("brim_width", json!("5")),
            ],
        );
    }
}

#[test]
fn belt_brim_shallow_lattice_matches_native() {
    super::compare(
        "brim-shallow-5deg",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("5")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
            ("enable_overhang_speed", json!("0")),
        ],
    );
}

#[test]
fn belt_brim_apron_prologue_and_bounds_match_native() {
    super::compare(
        "brim-prologue-1deg",
        true,
        false,
        &[
            ("belt_slice_rotation_angle", json!("1")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("5")),
            ("leading_brim_length", json!("30")),
            ("nozzle_temperature", json!(["210"])),
            ("nozzle_temperature_initial_layer", json!(["225"])),
            (
                "machine_start_gcode",
                json!(
                    ";PROBE_MIN={first_layer_print_min[0]},{first_layer_print_min[1]} MAX={first_layer_print_max[0]},{first_layer_print_max[1]}\nG90\nM104 S[nozzle_temperature_initial_layer]\n"
                ),
            ),
        ],
    );
}

#[test]
fn belt_brim_inner_only_hole_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt inner brim requires Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir().join("brim-inner-1deg-hole");
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(work.join("input.3mf"), two_holes_belt_inner()).unwrap();
    stages::run(
        runner.bin(),
        super::ARGS.map(str::to_owned),
        stages::Stage::Slice,
        &work,
    )
    .unwrap();
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    let result = super::run_ares(&work);
    assert!(
        result.status.success(),
        "Ares exited {} in {work:?}",
        result.status
    );
    let native = fs::read(work.join("native/plate_1.gcode")).unwrap();
    let actual = fs::read(out.join("plate_1.gcode")).unwrap();
    assert!(
        String::from_utf8_lossy(&native).contains(";TYPE:Brim\n"),
        "expected active inner belt brim in native output {work:?}"
    );
    golden::compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{error}; {work:?}"));
}

#[cfg(target_os = "linux")]
#[test]
fn belt_brim_prologue_empty_gap_warning_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt prologue gap warnings require Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir().join("brim-prologue-empty-gap");
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(
        work.join("input.3mf"),
        super::project(
            true,
            false,
            &[
                ("belt_slice_rotation_angle", json!("45")),
                ("brim_type", json!("outer_only")),
                ("brim_width", json!("5")),
                ("leading_brim_length", json!("20")),
            ],
            false,
        ),
    )
    .unwrap();
    let native = stages::run(
        runner.bin(),
        super::ARGS.map(str::to_owned),
        stages::Stage::Slice,
        &work,
    )
    .unwrap_err();
    assert_eq!(native.captured.unwrap().code, Some(156), "{work:?}");
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    assert_eq!(super::run_ares(&work).status.code(), Some(156), "{work:?}");

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
    assert_eq!(
        ares["sliced_plates"][0]["warning_message"], native["sliced_plates"][0]["warning_message"],
        "{work:?}"
    );
}

fn two_holes_belt_inner() -> Vec<u8> {
    let mut reader = ZipArchive::new(Cursor::new(TWO_HOLES)).unwrap();
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
                    ("belt_printer", "1"),
                    ("belt_slice_rotation", "x"),
                    ("belt_slice_rotation_angle", "1"),
                    ("gcode_remap_x", "rev_x"),
                    ("gcode_remap_y", "pos_z"),
                    ("gcode_remap_z", "pos_y"),
                    ("enable_arc_fitting", "disabled"),
                    ("brim_type", "inner_only"),
                    ("brim_width", "2"),
                    ("seam_position", "back"),
                    ("skirt_loops", "0"),
                    ("initial_layer_print_height", "0.3"),
                    ("layer_height", "0.3"),
                ] {
                    config.insert(key.to_owned(), json!(value));
                }
                bytes = serde_json::to_vec(&config).unwrap();
            }
            "3D/3dmodel.model" => {
                let model = String::from_utf8(bytes).unwrap();
                let target = "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"";
                assert_eq!(model.matches(target).count(), 1);
                bytes = model
                    .replacen(target, "transform=\"1 0 0 0 0.8 0 0 0 2 0 0 0\"", 1)
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
