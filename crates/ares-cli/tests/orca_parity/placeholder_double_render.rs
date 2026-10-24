//! PlaceholderParser renders every double through one precision-six stream.

#[test]
fn small_double_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-double-render",
            &["PlaceholderParser.cpp:302-324::expr::to_string/TYPE_DOUBLE"],
        ),
        "small-double-target",
        include_bytes!(
            "../../../../tests/parity/placeholder-double-render/small-double-target.3mf"
        ),
        "d5705287bf8ff2f47d1186e77f529e755e9b9db5be12aa4b9e533349a9e42739",
        (false, |_, _| {}),
    );
}

#[test]
fn integral_double_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-double-render",
            &["PlaceholderParser.cpp:302-324::expr::to_string/TYPE_DOUBLE"],
        ),
        "integral-double-target",
        include_bytes!(
            "../../../../tests/parity/placeholder-double-render/integral-double-target.3mf"
        ),
        "d04abfff556d4a4bb5d0eb366803eb74f723f24f763d25754bfcdf00874a1ee8",
        (false, |_, _| {}),
    );
}

#[test]
fn half_double_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-double-render",
            &["PlaceholderParser.cpp:302-324::expr::to_string/TYPE_DOUBLE"],
        ),
        "half-double-control",
        include_bytes!(
            "../../../../tests/parity/placeholder-double-render/half-double-control.3mf"
        ),
        "afc80845229f5428aad1ab7cf04eaa69716e8fd0aa61e438165fa60ec2983e58",
        (false, |_, _| {}),
    );
}
