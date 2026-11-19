use std::fs;

use crate::{
    OrcaRunner, golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &[
    "GCode/SeamPlacer.cpp:196-199,850-860,957-978,1176-1268",
    "GCode/SeamPlacer.cpp:1339-1350,1498-1627,1802-1818,1883-2081,2150-2204",
];

fn native(printer: &str, variant: &str) -> NativeCase {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-aligned-back")
            .join(&file),
    )
    .unwrap();
    NativeCase::slice(
        &OrcaRunner::from_env().unwrap().unwrap(),
        &project,
        &format!("seam-paint-aligned-back-{label}-"),
    )
}

fn first_inner_wall_approach(gcode: &str) -> &str {
    let mut approach = None;
    for line in gcode.lines() {
        if matches!(line, ";TYPE:Inner wall" | "; FEATURE: Inner wall") {
            return approach.expect("Native must travel to its first inner wall");
        }
        if line.starts_with("G1 X") && !line.split_whitespace().any(|word| word.starts_with('E')) {
            approach = Some(line);
        }
    }
    panic!("Native must print an inner wall");
}

#[test]
fn aligned_back_native_paint_changes_enforced_and_blocked_walls() {
    for printer in ["x1c", "k1", "mk4"] {
        let plain = native(printer, "plain");
        let front = native(printer, "front");
        let rear = native(printer, "rear");
        let both = native(printer, "both");
        let side = native(printer, "side");
        let plain_approach = first_inner_wall_approach(&plain.native);
        for painted in [&front, &rear, &both] {
            assert_ne!(first_inner_wall_approach(&painted.native), plain_approach);
        }
        assert!(
            golden::first_difference(
                &golden::normalize_one_generator_line(
                    plain.native.as_bytes(),
                    golden::GeneratorKind::Orca,
                )
                .unwrap(),
                &golden::normalize_one_generator_line(
                    side.native.as_bytes(),
                    golden::GeneratorKind::Orca,
                )
                .unwrap(),
            )
            .is_none()
        );
    }
}

fn compare(printer: &str, variant: &str) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-aligned-back")
            .join(&file),
    )
    .unwrap();
    let (digest, _) = include_str!("../../../../tests/parity/seam-paint-aligned-back/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find(|(_, name)| *name == file)
        .unwrap();
    native_stream_case::check(
        ("seam-paint-aligned-back", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

macro_rules! printer_cases {
    ($printer:ident, $label:literal) => {
        mod $printer {
            use super::compare;

            #[test]
            fn unpainted() {
                compare($label, "plain");
            }
            #[test]
            fn front_enforcer() {
                compare($label, "front");
            }
            #[test]
            fn rear_blocker() {
                compare($label, "rear");
            }
            #[test]
            fn both_annotations() {
                compare($label, "both");
            }
            #[test]
            fn side_paint() {
                compare($label, "side");
            }
        }
    };
}

printer_cases!(x1c, "x1c");
printer_cases!(k1, "k1");
printer_cases!(mk4, "mk4");
