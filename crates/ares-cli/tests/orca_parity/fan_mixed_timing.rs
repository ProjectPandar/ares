//! FanMover.cpp:284-305 / FanMover.hpp:19,46,57 mixed-precision timing.

#[test]
fn fan_mixed_timing_speedup_zero_control() {
    crate::runner::native_stream_case::check(
        (
            "fan-mixed-timing",
            &["FanMover.cpp:284-305", "FanMover.hpp:19,46,57"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/fan-mixed-timing/control.3mf"),
        "b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298",
        (false, |_, _| {}),
    );
}

#[test]
fn fan_mixed_timing_speedup_one_target() {
    crate::runner::native_stream_case::check(
        (
            "fan-mixed-timing",
            &["FanMover.cpp:284-305", "FanMover.hpp:19,46,57"],
        ),
        "target",
        include_bytes!("../../../../tests/parity/fan-mixed-timing/target.3mf"),
        "98149dd837288535426ca78195dce3451f24fc3dd0906e783e8526d9ae18e084",
        (false, |_, _| {}),
    );
}
