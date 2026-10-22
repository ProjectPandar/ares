//! Placeholder regex comparison matches the whole subject, not a substring.

#[test]
fn placeholder_regex_full_match_whole_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-regex-full-match",
            &["PlaceholderParser.cpp:699-704"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/placeholder-regex-full-match/control.3mf"),
        "03a3282d12e670df11f7e690e1dea0b80a36958427b38e2af535bef850b7bae5",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_regex_full_match_substring_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-regex-full-match",
            &["PlaceholderParser.cpp:699-704"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/placeholder-regex-full-match/positive.3mf"),
        "a394d6dc4525cd8f7d60d7eac592ff0160843181b8a1dc6587354b569a82868e",
        (false, |_, _| {}),
    );
}
