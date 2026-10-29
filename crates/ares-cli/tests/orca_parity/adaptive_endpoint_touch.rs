//! `FillAdaptive.cpp:815-880,939-954,1074-1110` endpoint-touch handling.

#[test]
fn orca_parity_adaptive_endpoint_touch_a1() {
    crate::runner::native_stream_case::check(
        (
            "adaptive-endpoint-touch",
            &[
                "Fill/FillAdaptive.cpp:815-880",
                "Fill/FillAdaptive.cpp:939-954",
                "Fill/FillAdaptive.cpp:1074-1110",
            ],
        ),
        "a1-adaptivecubic40",
        include_bytes!("../../../../tests/parity/adaptive-endpoint-touch/a1-adaptivecubic40.3mf"),
        "9667257ee00c05b082b0f5affb823def7c31dac5c0169d03da20c70890f84f21",
        (false, |_, _| {}),
    );
}
