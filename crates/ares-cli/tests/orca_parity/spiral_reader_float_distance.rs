//! Spiral reader float XY distance feeds cumulative finishing-flow progress.

#[test]
fn spiral_reader_float_distance_full_finish_control() {
    crate::runner::native_stream_case::check(
        (
            "spiral-reader-float-distance",
            &[
                "GCodeReader.hpp:47-51",
                "GCode/SpiralVase.cpp:84-96/143-159",
            ],
        ),
        "control",
        include_bytes!("../../../../tests/parity/spiral-reader-float-distance/control.3mf"),
        "833b90212289240a2bb655394129d054726fcb8877842f6cdc1264e06863ef7b",
        (false, |_, _| {}),
    );
}

#[test]
fn spiral_reader_float_distance_zero_finish_positive() {
    crate::runner::native_stream_case::check(
        (
            "spiral-reader-float-distance",
            &[
                "GCodeReader.hpp:47-51",
                "GCode/SpiralVase.cpp:84-96/143-159",
            ],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/spiral-reader-float-distance/positive.3mf"),
        "58483c5d001e4d491f1a40e4db76a99e41933fad9c3ee9bd6da607dea2a4d196",
        (false, |_, _| {}),
    );
}
