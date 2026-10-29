//! SpiralVase entry/final overlap: independent state, extrusion priority, passthrough.

#[test]
fn spiral_overlap_transition_nonspiral_control() {
    crate::runner::native_stream_case::check(
        (
            "spiral-overlap-transition",
            &["GCode/SpiralVase.cpp:122-123/153-166/206-215"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/spiral-overlap-transition/control.3mf"),
        "7a476876c3242aa7936de75e685b2c3292a32c678bf53f994b36d6c1e2308b77",
        (false, |_, _| {}),
    );
}

#[test]
fn spiral_overlap_transition_first_body_is_final_positive() {
    crate::runner::native_stream_case::check(
        (
            "spiral-overlap-transition",
            &["GCode/SpiralVase.cpp:122-123/153-166/206-215"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/spiral-overlap-transition/positive.3mf"),
        "d35033bde4712fd6befa3b7341c172901e654608be904cab43757cdcf6890621",
        (false, |_, _| {}),
    );
}
