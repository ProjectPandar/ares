//! GCodeProcessor.cpp:5347-5355 short-circuit dwell argument public streams.

#[test]
fn m400_valid_seconds_suppresses_milliseconds() {
    crate::runner::native_stream_case::check(
        ("m400-priority", &["GCode/GCodeProcessor.cpp:5347-5355"]),
        "dual-target",
        include_bytes!("../../../../tests/parity/m400-priority/m400-dual-target.3mf"),
        "27e1d3ec0b3276a37ffe8bf7e9d8e3693fa9ccc0a7167f1a1fe8314db2de2d7e",
        (false, |_, _| {}),
    );
}

#[test]
fn m400_seconds_only_control() {
    crate::runner::native_stream_case::check(
        ("m400-priority", &["GCode/GCodeProcessor.cpp:5347-5355"]),
        "seconds-control",
        include_bytes!("../../../../tests/parity/m400-priority/m400-s-control.3mf"),
        "827b8c6bf7451a18f5ebc47e3272291b3eadb5490d69de199cf041f9a20be442",
        (false, |_, _| {}),
    );
}

#[test]
fn g4_dual_argument_neighbor_control() {
    crate::runner::native_stream_case::check(
        ("m400-priority", &["GCode/GCodeProcessor.cpp:4848-4856"]),
        "g4-control",
        include_bytes!("../../../../tests/parity/m400-priority/g4-dual-control.3mf"),
        "c8047d059449bc8ab069898c2849f98d3f4c8606cd6578fd09e04ec9345c7c37",
        (false, |_, _| {}),
    );
}
