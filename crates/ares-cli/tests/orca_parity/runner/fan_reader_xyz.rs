//! FanMover.cpp:291-304 → GCodeReader raw XYZ deltas without G90/G91 state.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "fan-reader-xyz",
    &[
        "GCode/FanMover.cpp:291-304",
        "GCodeReader.hpp:43-45,129-135",
        "GCodeReader.cpp:110-123",
    ],
);

#[test]
fn fan_reader_xyz_disabled_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/fan-reader-xyz/control.3mf"),
        "06a72c88130a9c73bf6a4ef04ec54d7e2e22a85acd4220283b660513333bb87e",
        (false, |_, _| {}),
    );
}

#[test]
fn fan_reader_xyz_enabled_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/fan-reader-xyz/target.3mf"),
        "ed8a8a1c5fbed8a032dfbe222d2a1f4bbaa06ac79607fb0b5283bce25d071d04",
        (false, |_, _| {}),
    );
}
