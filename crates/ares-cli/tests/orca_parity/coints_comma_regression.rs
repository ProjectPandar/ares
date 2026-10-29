//! Registered Ints vector delimiters are split before integer leaf conversion.

#[test]
fn legal_coints_comma_list_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "coints-comma-regression",
            &[
                "Config.hpp:995-1006,1023-1031,1045-1068 ConfigOptionInts vector",
                "PlaceholderParser.cpp:932-945,1212 integer vector expansion",
            ],
        ),
        "seam-variant",
        include_bytes!("../../../../tests/parity/coints-comma-regression/seam-variant.3mf"),
        "3e392f7a34ea2b869722aa768ff729b51d1bf0c99d9f1be708671e618f06eebf",
        (false, |_, _| {}),
    );
}
