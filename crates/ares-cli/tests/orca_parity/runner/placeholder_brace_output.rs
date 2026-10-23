//! PlaceholderParser.cpp:2173,532-537,302-327: brace result ostream conversion.

#[test]
fn placeholder_brace_output_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-brace-output",
            &[
                "PlaceholderParser.cpp:2173",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-brace-output/control.3mf"),
        "c706bfee64a4e0bb504ecb431c6053ff429dac8ea0b8a3c62432c054e2401afa",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_brace_output_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-brace-output",
            &[
                "PlaceholderParser.cpp:2173",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-brace-output/positive.3mf"),
        "f092166be2d66e1c9f916fc8a822bdf9a1170bd60fcd6c0679920c85ebbc3364",
        (false, |_, _| {}),
    );
}
