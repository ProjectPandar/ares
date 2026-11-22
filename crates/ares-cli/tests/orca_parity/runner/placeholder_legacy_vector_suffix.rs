//! PlaceholderParser.cpp:1037-1077,2364-2375: standalone legacy vector suffixes.

use super::placeholder_deferred_misuse::assert_rejection_matches_native;
use crate::runner::native_stream_case;

const BOUNDARY: (&str, &[&str]) = (
    "legacy-placeholder-vector-suffix",
    &[
        "PlaceholderParser.cpp:1037-1077",
        "PlaceholderParser.cpp:2364-2375",
    ],
);

#[test]
fn legacy_startup_resolves_exact_identifiers_before_trying_suffixes() {
    native_stream_case::check(
        BOUNDARY,
        "control",
        include_bytes!("../../../../../tests/parity/legacy-placeholder-vector-suffix/control.3mf"),
        "b4837ba8a239a1ad6fb3eac10474a8cf03be3b33cb381ed6b4568b5b90755712",
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == ";START_VALUES 220 220x0 220")
            );
        }),
    );
}

#[test]
fn legacy_startup_selects_decimal_suffix_and_reuses_the_first_value_out_of_range() {
    native_stream_case::check(
        BOUNDARY,
        "indexed",
        include_bytes!("../../../../../tests/parity/legacy-placeholder-vector-suffix/indexed.3mf"),
        "e7b4e4a2d92f44290a220daa5ac8bbf3e60db606fe69b62697635dc6dbeec68f",
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == ";START_VALUES 220 220x0 220 220")
            );
        }),
    );
}

#[test]
fn legacy_numeric_suffix_cannot_index_a_scalar() {
    assert_rejection_matches_native(
        "legacy-placeholder-vector-suffix/scalar",
        "; HEIGHT=[max_print_height_1]",
        "Trying to index a scalar variable",
    );
}
