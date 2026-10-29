//! PrintObjectSlice.cpp:194-222; PrintApply.cpp:342-395,958-1027.
//! Release NDEBUG keeps ordinary ranged slicing for multi-range spiral input.
use crate::{golden, runner::native_stream_case};

const SOURCE: &[u8] =
    include_bytes!("../../../../../tests/parity/spiral-hole-projected-split/spiral.3mf");
const SOURCE_SHA: &str = "7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27";
const UPSTREAM: &[&str] = &[
    "PrintObjectSlice.cpp:194-222",
    "PrintApply.cpp:342-395,958-1027",
];

#[test]
fn spiral_layer_ranges_c0_single_default_matches_native_api_cli() {
    check("C0", SOURCE, SOURCE_SHA, 51, 0);
}

#[test]
fn spiral_layer_ranges_r1_active_multi_matches_native_api_cli() {
    check(
        "R1",
        include_bytes!("../../../../../tests/parity/spiral-layer-ranges/active.3mf"),
        "00cf2cd954a57cc4be34bb5eeca90b41fbe92cf017e75fcf51f3f29d0d116133",
        50,
        102,
    );
}

#[test]
fn spiral_layer_ranges_r2_ignored_override_matches_native_api_cli() {
    check(
        "R2",
        include_bytes!("../../../../../tests/parity/spiral-layer-ranges/ignored.3mf"),
        "32d081707888f472bd00f127a78e56514cb0351bf5c9ba480f08c9ac217ba568",
        51,
        0,
    );
}

fn check(label: &str, project: &[u8], sha: &str, layers: usize, speed_words: usize) {
    native_stream_case::check(
        ("spiral-layer-ranges", UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for case in [reference, replay] {
                assert!(case.native.contains("; CONFIG_BLOCK_END"));
                assert!(case.native.contains("; spiral_mode = 1\n"));
                assert_eq!(
                    case.native
                        .lines()
                        .filter(|line| line.starts_with(";Z:"))
                        .count(),
                    layers,
                    "Native physical layer schedule: {:?}",
                    case.root
                );
                assert_eq!(
                    case.native
                        .lines()
                        .filter(|line| line.split_whitespace().any(|word| word == "F1020"))
                        .count(),
                    speed_words,
                    "Native active/inactive 17mm/s override: {:?}",
                    case.root
                );
            }
            if label == "R2" {
                let (control, _) = native_stream_case::qualify(
                    ("spiral-layer-ranges", UPSTREAM),
                    "R2-control",
                    SOURCE,
                    SOURCE_SHA,
                    false,
                );
                // R2 proves admission of ignored settings, not range activity.
                let control = golden::normalize_one_generator_line(
                    control.native.as_bytes(),
                    golden::GeneratorKind::Orca,
                )
                .unwrap();
                let ignored = golden::normalize_one_generator_line(
                    reference.native.as_bytes(),
                    golden::GeneratorKind::Orca,
                )
                .unwrap();
                assert_eq!(
                    golden::first_difference(&control, &ignored).map(|diff| diff.to_string()),
                    None,
                    "R2 Native stream must equal C0"
                );
            }
        }),
    );
}
