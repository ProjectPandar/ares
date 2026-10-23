//! FanMover-generated fan comments and raw split-line separator preservation.
use super::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "fan-verbose-text",
    &[
        "GCode.cpp:2038",
        "GCodeWriter.cpp:1095-1130",
        "GCode/FanMover.cpp:75-87,124-212,231-236,398-418",
    ],
);

#[test]
fn fan_verbose_text_disabled_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/fan-verbose-text/control.3mf"),
        "ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632",
        (false, |_, _| {}),
    );
}

#[test]
fn fan_verbose_text_kickstart_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/fan-verbose-text/target.3mf"),
        "42123da0f4fd38bf64a1ea498e9985640816f952b16e170fbc0a63817aa2ce7f",
        (false, |_, _| {}),
    );
}
