//! PlaceholderParser.cpp regex_match uses byte subjects and lets dots span line feeds.

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
fn placeholder_regex_dot_matches_newline() {
    if crate::runner::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    crate::runner::native_stream_case::check(
        (
            "placeholder-regex-byte-semantics",
            &["PlaceholderParser.cpp:754-781"],
        ),
        "multiline-notes",
        include_bytes!(
            "../../../../tests/parity/placeholder-regex-byte-semantics/multiline-notes.3mf"
        ),
        "89508b0e02823a9502aad4f5df937096c2e1c2b68bc2b39f2200288d72691a06",
        (false, |first, replay| {
            for native in [first, replay] {
                assert!(
                    native
                        .native
                        .lines()
                        .any(|line| line == ";REGEX_DOT_NEWLINE true"),
                    "Native template did not match the multiline note: {:?}",
                    native.root
                );
            }
        }),
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
