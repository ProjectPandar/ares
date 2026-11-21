use std::fs;

use crate::{
    OrcaRunner, golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &["GCode/SeamPlacer.cpp:739-860,1227-1275,1366-1444,2181-2200"];

fn input(printer: &str, variant: &str) -> (String, Vec<u8>, &'static str) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-random")
            .join(&file),
    )
    .unwrap();
    let (digest, _) = include_str!("../../../../tests/parity/seam-paint-random/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find(|(_, name)| *name == file)
        .unwrap();
    (label, project, digest)
}

fn compare(printer: &str, variant: &str) {
    let (label, project, digest) = input(printer, variant);
    native_stream_case::check(
        ("seam-paint-random", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

fn native(runner: &OrcaRunner, printer: &str, variant: &str) -> String {
    let (label, project, _) = input(printer, variant);
    NativeCase::slice(runner, &project, &format!("random-wall-approach-{label}-")).native
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
fn random_native_paint_changes_wall_approaches() {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let normalize = |gcode: &str| {
        golden::normalize_one_generator_line(gcode.as_bytes(), golden::GeneratorKind::Orca).unwrap()
    };
    for printer in ["x1c", "k1", "mk4"] {
        let plain = native(&runner, printer, "plain");
        let front = native(&runner, printer, "front");
        let both = native(&runner, printer, "both");
        let rear = native(&runner, printer, "rear");
        let side = native(&runner, printer, "side");
        for (variant, painted) in [
            ("front", &front),
            ("rear", &rear),
            ("both", &both),
            ("side", &side),
        ] {
            assert_ne!(
                first_inner_wall_approach(painted),
                first_inner_wall_approach(&plain),
                "{printer} {variant} must change the first inner-wall approach"
            );
        }
        assert!(
            golden::first_difference(&normalize(&front), &normalize(&both)).is_none(),
            "{printer} rear blocker must not change output with the front enforcer already active"
        );
    }
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
