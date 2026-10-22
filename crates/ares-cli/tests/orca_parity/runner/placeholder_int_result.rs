//! PlaceholderParser.cpp:353-362,494-510: int returns integer for arithmetic.

#[test]
fn placeholder_int_result_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-int-result",
            &[
                "PlaceholderParser.cpp:353-362",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-int-result/control.3mf"),
        "f48720d0e82266c370b020b69218e2a57a5318e9eb8ff3ec51f0b7030eb7b5d9",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_int_result_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-int-result",
            &[
                "PlaceholderParser.cpp:353-362",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-int-result/positive.3mf"),
        "723e068e512214d33ba7362763edf31d4e12f4a1ac180fd45d7ebd54522b65b8",
        (false, |_, _| {}),
    );
}
