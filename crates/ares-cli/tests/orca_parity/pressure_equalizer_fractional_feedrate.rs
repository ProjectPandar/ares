//! CoolingBuffer.cpp:983–1019: modal F words after pressure equalization.

use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "GCode/CoolingBuffer.cpp:894,983-1019",
    "GCode/PressureEqualizer.cpp:483-520,930-955",
];
const PLATE_RESTORE: &str =
    "G1 F9251.231\nG1 X105.754 Y99.355 E.00942\nG1 F9240\nG1 X105.704 Y99.657 E.00991\n";
const SPAN_REUSE: &str = "G1 F9540.608\nG1 X95.506 Y91.173 E.00703\nG1 X92.823 Y91.173 E.08416\n";

fn check(label: &str, slope: &str, project: &[u8], sha: &str, transition: Option<&str>) {
    let configured_slope = format!("; max_volumetric_extrusion_rate_slope = {slope}");
    native_stream_case::check(
        ("pressure-equalizer-fractional-feedrate", UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                assert!(native.lines().any(|line| line == configured_slope));
                if let Some(transition) = transition {
                    assert!(
                        native.contains(transition),
                        "Native modal F transition: {label}"
                    );
                }
            }
        }),
    );
}

macro_rules! case {
    ($name:ident, $label:literal, $slope:literal, $sha:literal, $transition:expr) => {
        #[test]
        fn $name() {
            check(
                $label,
                $slope,
                include_bytes!(concat!(
                    "../../../../tests/parity/pressure-equalizer-fractional-feedrate/",
                    $label,
                    ".3mf"
                )),
                $sha,
                $transition,
            );
        }
    };
}

case!(
    mk4_plate_restores_feedrate_after_fractional_f_slope_13,
    "mk4-plate-13",
    "13",
    "d69e59e7954d849d6de977f48a3b5e19c95d7c11efb1074546bc7dc83e09cc8b",
    Some(PLATE_RESTORE)
);
case!(
    mk4_plate_restores_feedrate_after_fractional_f_slope_280,
    "mk4-plate-280",
    "280",
    "15ef3cbfcc56dcdb3678df71a3c0d8fd6c0d87e1566ab85c901279db34043901",
    Some(PLATE_RESTORE)
);
case!(
    mk4_span_skips_repeated_integer_feedrate_slope_13,
    "mk4-span-13",
    "13",
    "ec17745854cdbe6168ae93c251b160378917091701690f91d0bfb8b409b23f52",
    Some(SPAN_REUSE)
);
case!(
    mk4_span_skips_repeated_integer_feedrate_slope_280,
    "mk4-span-280",
    "280",
    "6d6324bef145f5cc7729d088ab9f693ea6a82954af77819248fb6faec0b907f4",
    Some(SPAN_REUSE)
);
case!(
    k1_plate_preserves_pressure_equalized_output,
    "k1-plate-280",
    "280",
    "2ad1c53ed3bf50b082845849c12219b52fcb0aa5ee282afddf73917e611716ca",
    None
);
