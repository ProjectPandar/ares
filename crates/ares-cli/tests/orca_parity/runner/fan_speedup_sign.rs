//! GCode.cpp:3732-3740,3830-3838 → FanMover.hpp:63-70 constructor contract.
use super::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "fan-speedup-sign",
    &[
        "GCode.cpp:3732-3740",
        "GCode.cpp:3830-3838",
        "GCode/FanMover.hpp:63-70",
    ],
);

#[test]
fn fan_speedup_sign_zero_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/fan-layer-tag/control.3mf"),
        "17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2",
        (false, |_, _| {}),
    );
}

#[test]
fn fan_speedup_sign_negative_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/fan-speedup-sign/target.3mf"),
        "19507db8da1597d32a348f299edaa8d24a5e16aa203b6f3150da6ca308dc049d",
        (false, |_, _| {}),
    );
}
