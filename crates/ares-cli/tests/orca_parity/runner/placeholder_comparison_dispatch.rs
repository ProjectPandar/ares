//! PlaceholderParser.cpp:558-600 comparison dispatch and :302-327 to_string.

#[test]
fn placeholder_comparison_dispatch_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-comparison-dispatch",
            &[
                "PlaceholderParser.cpp:558-600",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-comparison-dispatch/control.3mf"),
        "574dcacc337821120b9c2d5b563b44747a29c919e02d53019e52335c842480fe",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_comparison_dispatch_leading_zero() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-comparison-dispatch",
            &[
                "PlaceholderParser.cpp:558-600",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "leading_zero",
        include_bytes!(
            "../../../../../tests/parity/placeholder-comparison-dispatch/leading-zero.3mf"
        ),
        "8f0568f2b982fb57f64f1fc53e7d41f0f8c1867c1a82498fee4d1579360b08e8",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_comparison_dispatch_double_string() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-comparison-dispatch",
            &[
                "PlaceholderParser.cpp:558-600",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "double_string",
        include_bytes!(
            "../../../../../tests/parity/placeholder-comparison-dispatch/double-string.3mf"
        ),
        "f57fc484e6b1f9e22b33eb504d1f209b5d4babcd796c61b82d61cb200a62b882",
        (false, |_, _| {}),
    );
}
