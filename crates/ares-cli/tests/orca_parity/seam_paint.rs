use std::fs;

use crate::{
    OrcaRunner, golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &[
    "Format/bbs_3mf.cpp:357,4041,5389-5390,7773-7780",
    "GCode/SeamPlacer.cpp:601-979,1077-1156,1176-1275,1498-1627,2005-2083",
];
const VARIANTS: [&str; 5] = ["plain", "front", "rear", "both", "side"];

fn input(printer: &str, variant: &str) -> (String, Vec<u8>, String) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(repo_root().join("tests/parity/seam-paint").join(&file)).unwrap();
    let (digest, _) = include_str!("../../../../tests/parity/seam-paint/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find(|(_, name)| *name == file)
        .unwrap();
    (label, project, digest.to_owned())
}

fn native(printer: &str, variant: &str) -> NativeCase {
    let (label, project, digest) = input(printer, variant);
    use crate::runner::native_api_cli::sha;
    assert_eq!(sha(&project), digest);
    NativeCase::slice(
        &OrcaRunner::from_env().unwrap().unwrap(),
        &project,
        &format!("seam-paint-{label}-"),
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
fn seam_paint_native_changes_front_and_rear_but_not_side() {
    for printer in ["x1c", "k1", "mk4"] {
        let [plain, front, rear, both, side] = VARIANTS.map(|variant| native(printer, variant));
        let plain_approach = first_inner_wall_approach(&plain.native);
        for painted in [front, rear, both] {
            assert_ne!(
                first_inner_wall_approach(&painted.native),
                plain_approach,
                "{printer}: {:?}",
                painted.root
            );
        }
        let normalize = |native: &NativeCase| {
            golden::normalize_one_generator_line(
                native.native.as_bytes(),
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
    let (label, project, digest) = input(printer, variant);
    native_stream_case::check(
        ("seam-paint", UPSTREAM),
        &label,
        &project,
        &digest,
        (false, |_, _| {}),
    );
}

#[test]
fn deferred_seam_paint_modes_reject_without_output() {
    let (_, project, _) = input("x1c", "front");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let overrides =
        ares_core::ConfigOverrides::from_json(br#"{"seam_position":"nearest","spiral_mode":"1"}"#)
            .unwrap();
    let result = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::default(),
        None,
        &overrides,
        ares_core::SlicerContext::new(0),
    ));
    assert!(
        matches!(result, Err(ares_core::SliceError::InvalidInput(ref message))
        if message.contains("seam painting is outside the supported")),
        "{result:?}"
    );
}

macro_rules! printer_cases {
    ($printer:literal, $plain:ident, $front:ident, $rear:ident, $both:ident, $side:ident) => {
        #[test]
        fn $plain() {
            compare($printer, "plain");
        }
        #[test]
        fn $front() {
            compare($printer, "front");
        }
        #[test]
        fn $rear() {
            compare($printer, "rear");
        }
        #[test]
        fn $both() {
            compare($printer, "both");
        }
        #[test]
        fn $side() {
            compare($printer, "side");
        }
    };
}

printer_cases!(
    "x1c",
    x1c_unpainted,
    x1c_front_enforcer,
    x1c_rear_blocker,
    x1c_both_annotations,
    x1c_side_paint
);
printer_cases!(
    "k1",
    k1_unpainted,
    k1_front_enforcer,
    k1_rear_blocker,
    k1_both_annotations,
    k1_side_paint
);
printer_cases!(
    "mk4",
    mk4_unpainted,
    mk4_front_enforcer,
    mk4_rear_blocker,
    mk4_both_annotations,
    mk4_side_paint
);
