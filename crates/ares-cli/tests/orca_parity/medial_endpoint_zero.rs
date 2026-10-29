//! ExPolygon.cpp endpoint extension consumes Eigen's zero-preserving normalized.

#[test]
fn duplicate_medial_endpoint_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "medial-endpoint-zero",
            &[
                "ExPolygon.cpp:295-313",
                "Eigen/src/Core/Dot.h:MatrixBase::normalized",
            ],
        ),
        "beam1-target",
        include_bytes!("../../../../tests/parity/medial-endpoint-zero/beam1-target.3mf"),
        "a9ee974eec98a58a499dea4b508dc186cc32708d8364faff4098e0257eec8775",
        (true, |_, _| {}),
    );
}

#[test]
fn disabled_interlocking_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "medial-endpoint-zero",
            &[
                "ExPolygon.cpp:295-313",
                "Eigen/src/Core/Dot.h:MatrixBase::normalized",
            ],
        ),
        "beam0-control",
        include_bytes!("../../../../tests/parity/medial-endpoint-zero/beam0-control.3mf"),
        "42309a59caea9f564dd566e2c0aa8914d5d22bc21fa127accbe4574408b7c523",
        (true, |_, _| {}),
    );
}
