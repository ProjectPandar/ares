use std::{fs, path::Path};

use crate::{
    OrcaRunner, golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &[
    "TriangleSelector.cpp:1474-1493,1540-1666,1717-1799,1813-1949",
    "GCode/SeamPlacer.cpp:1114-1133",
];

fn load(printer: &str, variant: &str) -> (String, Vec<u8>, &'static str) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let data = include_str!("../../../../tests/parity/seam-paint-split/SHA256SUMS");
    let digest = data
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find_map(|(digest, name)| (file == name).then_some(digest))
        .unwrap();
    let dir = if variant == "plain" {
        "seam-paint"
    } else {
        "seam-paint-split"
    };
    let project = fs::read(
        repo_root()
            .join(Path::new("tests/parity"))
            .join(dir)
            .join(file),
    )
    .unwrap();
    (label, project, digest)
}

fn native(printer: &str, variant: &str) -> NativeCase {
    let (label, project, digest) = load(printer, variant);
    assert_eq!(crate::runner::native_api_cli::sha(&project), digest);
    NativeCase::slice(
        &OrcaRunner::from_env().unwrap().unwrap(),
        &project,
        &format!("seam-paint-split-{label}-"),
    )
}

fn first_inner_wall_approach(gcode: &str) -> &str {
    gcode
        .lines()
        .take_while(|line| !matches!(*line, ";TYPE:Inner wall" | "; FEATURE: Inner wall"))
        .filter(|line| {
            line.starts_with("G1 X") && !line.split_whitespace().any(|word| word.starts_with('E'))
        })
        .last()
        .unwrap()
}

#[test]
fn subdivided_front_patch_moves_native_seam_but_side_control_does_not() {
    for printer in ["x1c", "k1", "mk4"] {
        let plain = native(printer, "plain");
        let front = native(printer, "front");
        let side = native(printer, "side");
        assert_ne!(
            first_inner_wall_approach(&plain.native),
            first_inner_wall_approach(&front.native),
            "{printer}: {:?}",
            front.root
        );
        let normalize = |output: &NativeCase| {
            golden::normalize_one_generator_line(
                output.native.as_bytes(),
                golden::GeneratorKind::Orca,
            )
            .unwrap()
        };
        assert!(
            golden::first_difference(&normalize(&plain), &normalize(&side)).is_none(),
            "{printer}: {:?}",
            side.root
        );
    }
}

fn compare(printer: &str, variant: &str) {
    let (label, project, digest) = load(printer, variant);
    native_stream_case::check(
        ("seam-paint-split", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

macro_rules! printer_cases {
    ($printer:literal, $front:ident, $side:ident) => {
        #[test]
        fn $front() {
            compare($printer, "front");
        }
        #[test]
        fn $side() {
            compare($printer, "side");
        }
    };
}

printer_cases!(
    "x1c",
    x1c_subdivided_front_patch,
    x1c_subdivided_side_control
);
printer_cases!("k1", k1_subdivided_front_patch, k1_subdivided_side_control);
printer_cases!(
    "mk4",
    mk4_subdivided_front_patch,
    mk4_subdivided_side_control
);
