//! PlaceholderParser.cpp:311-324: expression conversion preserves negative zero.

#[test]
fn placeholder_signed_zero_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-signed-zero",
            &["PlaceholderParser.cpp:311-324"],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-signed-zero/control.3mf"),
        "54f92e24f30e9f37678ab021feb471a44380fc98c3e7b801e767572bbcb3213a",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_signed_zero_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-signed-zero",
            &["PlaceholderParser.cpp:311-324"],
        ),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-signed-zero/positive.3mf"),
        "ffdce8681ab60c553167d27d9b26c43fcb43d73294f570c34b42078215763eb7",
        (false, |_, _| {}),
    );
}
