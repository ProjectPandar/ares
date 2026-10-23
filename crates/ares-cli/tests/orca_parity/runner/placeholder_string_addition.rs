//! PlaceholderParser.cpp:435-456: addition concatenates string operands.

#[test]
fn placeholder_string_addition_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-string-addition",
            &[
                "PlaceholderParser.cpp:435-456",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-string-addition/control.3mf"),
        "27563b597bd2092478777c526577e7816c64d747c1b597f3fa6fe17b861756a7",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_string_addition_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-string-addition",
            &[
                "PlaceholderParser.cpp:435-456",
                "PlaceholderParser.cpp:302-327",
            ],
        ),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-string-addition/positive.3mf"),
        "3d516226ce7f25db8b6db2f10e3a38c1674bafc599869c0bb43f0c8f5a6cca04",
        (false, |_, _| {}),
    );
}
