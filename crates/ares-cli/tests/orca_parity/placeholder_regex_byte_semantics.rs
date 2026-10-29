//! PlaceholderParser.cpp:699-704 regex_match byte semantics for UTF-8 subjects.

#[test]
fn placeholder_regex_byte_semantics_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-regex-byte-semantics",
            &["PlaceholderParser.cpp:699-704"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/placeholder-regex-byte-semantics/control.3mf"),
        "abab62dacf8e1905c7a809e331423d734873cda71789b5a189b5a388df0e2682",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_regex_byte_semantics_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-regex-byte-semantics",
            &["PlaceholderParser.cpp:699-704"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/placeholder-regex-byte-semantics/positive.3mf"),
        "5d7efb9e556d9c68ab711212a116b9817ffa56f8f33873342669915cb7fb3ef6",
        (false, |_, _| {}),
    );
}
