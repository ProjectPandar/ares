//! `BeltBackTransform`, `MachineFrameTransform`, `BeltKinematics`, and
//! `belt_frame_tilt_decouple` whole-G-code parity against OrcaSlicer v2.5.0.

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

const X_REMAP: [(&str, &str); 3] = [
    ("gcode_remap_x", "rev_x"),
    ("gcode_remap_y", "pos_z"),
    ("gcode_remap_z", "pos_y"),
];

const Y_REMAP: [(&str, &str); 3] = [
    ("gcode_remap_x", "pos_z"),
    ("gcode_remap_y", "pos_y"),
    ("gcode_remap_z", "pos_x"),
];

#[test]
fn x_axis_45_slice_30_decoupled_frame_matches_native_and_differs_from_coupled() {
    let Some((coupled, decoupled)) = compare_pair("x-45-coupled-vs-30", "x", "45", "30", &X_REMAP)
    else {
        return;
    };
    assert_ne!(
        coupled, decoupled,
        "decoupled 30 deg machine frame must change emitted coordinates relative to coupled 45 deg"
    );
}

#[test]
fn x_axis_30_slice_45_decoupled_frame_matches_native() {
    compare(
        "x-30-decoupled-45",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "30"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "45"),
        ],
        &X_REMAP,
    );
}

#[test]
fn y_axis_45_slice_30_decoupled_frame_matches_native_and_differs_from_coupled() {
    let Some((coupled, decoupled)) = compare_pair("y-45-coupled-vs-30", "y", "45", "30", &Y_REMAP)
    else {
        return;
    };
    assert_ne!(
        coupled, decoupled,
        "Y-axis decoupled 30 deg machine frame must change emitted coordinates relative to coupled 45 deg"
    );
}

#[test]
fn y_axis_30_slice_60_decoupled_frame_matches_native() {
    compare(
        "y-30-decoupled-60",
        false,
        &[
            ("belt_slice_rotation", "y"),
            ("belt_slice_rotation_angle", "30"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "60"),
        ],
        &Y_REMAP,
    );
}

#[test]
fn zero_and_sub_epsilon_decoupled_frame_angle_disables_machine_frame() {
    compare(
        "x-45-decoupled-0",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "0"),
        ],
        &X_REMAP,
    );
    compare(
        "x-45-decoupled-sub-epsilon",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "0.00005"),
        ],
        &X_REMAP,
    );
}

#[test]
fn negative_decoupled_frame_and_negative_slice_angles_match_native() {
    compare(
        "x-45-decoupled-neg30",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "-30"),
        ],
        &X_REMAP,
    );
    compare(
        "x-neg45-decoupled-30",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "-45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "30"),
        ],
        &X_REMAP,
    );
}

#[test]
fn translated_instance_with_decoupled_frame_preserves_forward_origin() {
    compare(
        "x-45-decoupled-30-shifted",
        true,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "30"),
        ],
        &X_REMAP,
    );
}

#[test]
fn right_angle_slice_and_fractional_decoupled_frame_match_native() {
    compare(
        "x-90-decoupled-30",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "90"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "30"),
        ],
        &X_REMAP,
    );
    compare(
        "x-45-decoupled-37-25",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "37.25"),
        ],
        &X_REMAP,
    );
}

#[test]
fn decoupled_angle_matching_slice_angle_matches_native() {
    compare(
        "x-45-decoupled-45",
        false,
        &[
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", "45"),
        ],
        &X_REMAP,
    );
}

fn compare_pair(
    name: &str,
    axis: &str,
    slice_angle: &str,
    frame_angle: &str,
    remap: &[(&str, &str); 3],
) -> Option<(Vec<u8>, Vec<u8>)> {
    let coupled = compare(
        &format!("{name}-coupled"),
        false,
        &[
            ("belt_slice_rotation", axis),
            ("belt_slice_rotation_angle", slice_angle),
            ("belt_frame_tilt_decouple", "0"),
            ("belt_frame_tilt_angle", frame_angle),
        ],
        remap,
    )?;
    let decoupled = compare(
        &format!("{name}-decoupled"),
        false,
        &[
            ("belt_slice_rotation", axis),
            ("belt_slice_rotation_angle", slice_angle),
            ("belt_frame_tilt_decouple", "1"),
            ("belt_frame_tilt_angle", frame_angle),
        ],
        remap,
    )?;
    Some((coupled, decoupled))
}

fn project(shift: bool, edits: &[(&str, &str)], remap: &[(&str, &str); 3]) -> Vec<u8> {
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
                config.insert("belt_printer".to_owned(), json!("1"));
                config.insert("enable_arc_fitting".to_owned(), json!("disabled"));
                for &(key, value) in remap.iter().chain(edits.iter()) {
                    config.insert(key.to_owned(), json!(value));
                }
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

fn compare(
    name: &str,
    shift: bool,
    edits: &[(&str, &str)],
    remap: &[(&str, &str); 3],
) -> Option<Vec<u8>> {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt frame tilt decouple requires Native OrcaSlicer");
        return None;
    };
    let work = runner.work_dir().join(name);
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(work.join("input.3mf"), project(shift, edits, remap)).unwrap();
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
        "Ares exited {} in {work:?}: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
    let native = fs::read(work.join("native/plate_1.gcode")).unwrap();
    let actual = fs::read(out.join("plate_1.gcode")).unwrap();
    golden::compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{error}; {work:?}"));
    Some(actual)
}
