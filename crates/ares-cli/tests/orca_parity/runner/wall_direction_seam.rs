//! GCode.cpp:5744-5775 and GCode/SeamPlacer.cpp:1500-1627.
//! Every non-spiral perimeter must use the source seam placement path,
//! including clockwise rectangular holes.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "wall-direction-seam",
    &[
        "GCode.cpp:5744-5758,5768-5775",
        "GCode/SeamPlacer.cpp:1500-1556",
        "GCode/SeamPlacer.cpp:1558-1627",
        "PerimeterGenerator.cpp:260-264,529-545",
    ],
);

#[test]
fn wall_direction_ccw_hole_control_matches_native_complete_output() {
    check(
        SOURCE,
        "ccw-control",
        include_bytes!("../../../../../tests/parity/wall-direction-seam/ccw-control.3mf"),
        "530ddd8fd603849dd5d701fa29627fa1c08ac42a323d097120290235b537e8a3",
        (false, |_, _| {}),
    );
}

#[test]
fn wall_direction_cw_hole_target_matches_native_complete_output() {
    check(
        SOURCE,
        "cw-target",
        include_bytes!("../../../../../tests/parity/wall-direction-seam/cw-target.3mf"),
        "3b0ac33c8c0c0149896bead08f0cbe48465dfa629a63ae79c1712af542ef2daa",
        (false, |_, _| {}),
    );
}
