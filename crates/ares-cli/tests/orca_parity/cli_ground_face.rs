#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use super::cli_project_transforms::outputs;
use crate::{
    golden::{GeneratorKind, compare_ordered_bytes_generator_only, normalize_one_generator_line},
    runner::{OrcaRunner, stages},
};

// An 8 × 12 × 16 mm box with its local origin at its center, initially resting on Z.
const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ground_box.3mf");
// Two touching parts share the bottom and both X planes. One upper corner is
// clipped by a triangular hull face with a 0.90 mm edge and area over 5 mm².
const MULTIPART: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ground_multipart.3mf");

#[test]
fn largest_face_lays_coplanar_parts_on_their_wide_side() {
    verify_project(MULTIPART, &["--ground-largest-face"], Some(&[]), 0);
}

#[test]
fn normal_near_a_short_edged_facet_lays_coplanar_parts_on_a_side() {
    verify_project(MULTIPART, &["--ground-face-normal", "2,-3,0"], Some(&[]), 0);
}

#[test]
fn largest_face_lays_the_box_on_its_wide_side() {
    verify(&["--ground-largest-face"], Some(&[]), 0);
}

#[test]
fn normal_selects_a_non_largest_face() {
    verify(&["--ground-face-normal", "0,2,0"], Some(&[]), 0);
}

#[test]
fn point_selects_a_face_after_volume_rotation() {
    verify(
        &["--rotate", "90", "--ground-face-point", "0,4,0"],
        Some(&["--rotate", "90"]),
        0,
    );
}

#[test]
fn point_before_volume_rotation_misses() {
    verify(
        &["--ground-face-point", "0,4,0", "--rotate", "90"],
        None,
        254,
    );
}

#[test]
fn point_on_plane_outside_outline_misses() {
    verify(&["--ground-face-point", "4,7,0"], None, 254);
}

#[test]
fn zero_normal_is_invalid() {
    verify(&["--ground-face-normal", "0,0,0"], None, 254);
}

#[test]
fn malformed_point_is_invalid() {
    verify(&["--ground-face-point", "4,0,0junk"], None, 254);
}

#[test]
fn last_largest_face_boolean_can_disable_grounding() {
    verify(
        &["--ground-largest-face", "--ground-largest-face=OFF"],
        None,
        0,
    );
}

fn verify(flags: &[&str], control: Option<&[&str]>, exit: i32) {
    verify_project(PROJECT, flags, control, exit);
}

fn verify_project(project: &[u8], flags: &[&str], control: Option<&[&str]>, exit: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-ground-face-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("project.3mf");
    fs::write(&input, project).unwrap();
    let input = input.to_str().unwrap();
    let (native, report) = run_native(&runner, &root.join("native"), input, flags, exit);
    let (replay, replay_report) =
        run_native(&runner, &root.join("native-replay"), input, flags, exit);
    assert_eq!(report, replay_report, "Native result replay: {root:?}");
    assert!(
        orca_bytes(&native) == orca_bytes(&replay),
        "Native G-code replay: {root:?}"
    );
    if let Some(control) = control {
        let (baseline, _) = run_native(&runner, &root.join("native-control"), input, control, 0);
        assert!(
            orca_bytes(&native) != orca_bytes(&baseline),
            "No change from grounding: {root:?}"
        );
    }

    let work = root.join("ares");
    fs::create_dir(&work).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command
        .current_dir(&work)
        .args(["slice", "-o", "plate_1.gcode"]);
    if let Some(catalog) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") {
        command.arg("--project-preset-catalog").arg(catalog);
    }
    let result = command.args(flags).arg(input).output().unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "Ares: {work:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let (actual, actual_report) = outputs(&work, exit);
    assert_eq!(report, actual_report, "result.json: {root:?}");
    if let (Some(native), Some(actual)) = (native, actual) {
        compare_ordered_bytes_generator_only(&native, &actual)
            .unwrap_or_else(|error| panic!("{root:?}: {error}"));
    }
}

fn orca_bytes(stream: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    stream
        .as_ref()
        .map(|bytes| normalize_one_generator_line(bytes, GeneratorKind::Orca).unwrap())
}

fn run_native(
    runner: &OrcaRunner,
    work: &Path,
    input: &str,
    flags: &[&str],
    exit: i32,
) -> (Option<Vec<u8>>, Vec<u8>) {
    fs::create_dir(work).unwrap();
    let args = flags
        .iter()
        .copied()
        .chain(["--slice", "0", "--outputdir", ".", input])
        .map(str::to_owned);
    let result = stages::run_live(runner.bin(), args, stages::Stage::Slice, work);
    assert_eq!(
        result.map_or_else(|error| error.captured.unwrap().code, |()| Some(0)),
        Some(exit),
        "Native: {work:?}"
    );
    outputs(work, exit)
}
