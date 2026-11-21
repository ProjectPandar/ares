use std::fs;

use crate::{
    OrcaRunner, golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &[
    "TriangleSelector.cpp:1492-1510,1855-1886",
    "Model.cpp:4138-4164",
    "GCode/SeamPlacer.cpp:829-859,1099-1153",
];

fn native_control(mode: &str, printer: &str, variant: &str) -> NativeCase {
    let (directory, prefix) = match mode {
        "aligned_back" => ("seam-paint-aligned-back", ""),
        "nearest" => ("seam-paint-nearest-rear", "nearest-"),
        "random" => ("seam-paint-random", ""),
        _ => unreachable!(),
    };
    let project = fs::read(repo_root().join(format!(
        "tests/parity/{directory}/{prefix}{printer}-{variant}.3mf"
    )))
    .unwrap();
    NativeCase::slice(
        &OrcaRunner::from_env().unwrap().unwrap(),
        &project,
        &format!("extended-palette-control-{mode}-{printer}-{variant}-"),
    )
}

fn inner_wall_approaches(gcode: &str) -> Vec<&str> {
    let mut approach = None;
    let mut walls = Vec::new();
    for line in gcode.lines() {
        if matches!(line, ";TYPE:Inner wall" | "; FEATURE: Inner wall") {
            walls.push(approach.expect("Native must travel to its inner walls"));
        }
        if line.starts_with("G1 X") && !line.split_whitespace().any(|word| word.starts_with('E')) {
            approach = Some(line);
        }
    }
    assert!(!walls.is_empty(), "Native must print an inner wall");
    walls
}

fn compare(mode: &str, printer: &str, variant: &str) {
    let label = format!("{mode}-{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-extended-palette")
            .join(&file),
    )
    .unwrap();
    let (digest, _) =
        include_str!("../../../../tests/parity/seam-paint-extended-palette/SHA256SUMS")
            .lines()
            .filter_map(|line| line.split_once("  "))
            .find(|(_, name)| *name == file)
            .unwrap();
    native_stream_case::check(
        ("seam-paint-extended-palette", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |reference, _| {
            let plain = native_control(mode, printer, "plain");
            let control = if variant == "enforcer-with-rear-state-32" {
                let front = native_control(mode, printer, "front");
                assert_ne!(
                    inner_wall_approaches(&front.native),
                    inner_wall_approaches(&plain.native),
                    "{label}: the real enforcer must change the Native wall approach"
                );
                front
            } else {
                plain
            };
            let normalized = |gcode: &str| {
                golden::normalize_one_generator_line(gcode.as_bytes(), golden::GeneratorKind::Orca)
                    .unwrap()
            };
            let difference = golden::first_difference(
                &normalized(&control.native),
                &normalized(&reference.native),
            );
            if let Some(difference) = difference {
                panic!("{label}: inert Native paint changed output: {difference}");
            }
        }),
    );
}

macro_rules! printer_cases {
    ($module:ident, $printer:literal, $mode:literal) => {
        mod $module {
            use super::compare;

            #[test]
            fn state_three_leaves_front_neutral() {
                compare($mode, $printer, "front-state-3");
            }
            #[test]
            fn state_four_leaves_rear_neutral() {
                compare($mode, $printer, "rear-state-4");
            }
            #[test]
            fn state_fifteen_leaves_split_front_neutral() {
                compare($mode, $printer, "split-front-state-15");
            }
            #[test]
            fn state_seventeen_leaves_front_neutral() {
                compare($mode, $printer, "front-state-17");
            }
            #[test]
            fn state_eighteen_leaves_rear_neutral() {
                compare($mode, $printer, "rear-state-18");
            }
            #[test]
            fn state_thirty_two_leaves_front_neutral() {
                compare($mode, $printer, "front-state-32");
            }
            #[test]
            fn state_thirty_two_can_cover_the_whole_box() {
                compare($mode, $printer, "all-state-32");
            }
            #[test]
            fn rear_state_thirty_two_preserves_front_enforcer() {
                compare($mode, $printer, "enforcer-with-rear-state-32");
            }
        }
    };
}

printer_cases!(aligned_back_x1c, "x1c", "aligned_back");
printer_cases!(aligned_back_k1, "k1", "aligned_back");
printer_cases!(aligned_back_mk4, "mk4", "aligned_back");
printer_cases!(nearest_x1c, "x1c", "nearest");
printer_cases!(nearest_k1, "k1", "nearest");
printer_cases!(nearest_mk4, "mk4", "nearest");
printer_cases!(random_x1c, "x1c", "random");
printer_cases!(random_k1, "k1", "random");
printer_cases!(random_mk4, "mk4", "random");
