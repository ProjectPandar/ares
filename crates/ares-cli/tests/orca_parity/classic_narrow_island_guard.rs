//! Beta `PerimeterGenerator.cpp:1718,1861-1863,1867`: narrow-island guard.

use crate::runner::native_stream_case;

#[test]
fn narrow_tube_skips_sandwich_matches_native_complete_output() {
    native_stream_case::check(
        (
            "classic-narrow-island-guard",
            &[
                "PerimeterGenerator.cpp::inner_wall_reach:1713-1718",
                "PerimeterGenerator.cpp::narrow_wall_hole_island:1861-1863",
                "PerimeterGenerator.cpp::sandwich eligibility guard:1867",
            ],
        ),
        "narrow-tube",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/classic-narrow-island-guard/narrow-tube.3mf"
        )),
        "01b634386258e99d65c8de97f2c47578917388c3b259a32128a5e09125bba372",
        (false, |reference, _| {
            for marker in [
                "; wall_generator = classic\n",
                "; wall_sequence = inner-outer-inner wall\n",
                "; wall_loops = 3\n",
                ";Z:0.4\n",
                ";TYPE:Outer wall\n",
                ";TYPE:Gap infill\n",
            ] {
                assert!(reference.native.contains(marker));
            }
        }),
    );
}
