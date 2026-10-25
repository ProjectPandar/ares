//! GCode.cpp per-layer FanMover filter and processor reserved layer-tag handoff.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "fan-layer-tag",
    &[
        "GCode.cpp:3727-3743,3830-3841,4619",
        "GCode/GCodeProcessor.cpp:64,87",
        "GCode/GCodeProcessor.hpp:375",
    ],
);

#[test]
fn fan_layer_tag_zero_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/fan-layer-tag/control.3mf"),
        "17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2",
        (false, |_, _| {}),
    );
}

#[test]
fn fan_layer_tag_positive_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/fan-layer-tag/target.3mf"),
        "a7ed84d65c690a3e50b7d8623e545fde6537f15ffdf43db209d940d3285361c3",
        (false, |_, _| {}),
    );
}
