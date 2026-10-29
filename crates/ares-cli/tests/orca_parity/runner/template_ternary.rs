//! Inactive ternary branches are parsed but not evaluated
//! (`PlaceholderParser.cpp:2215-2221,851-871`). The frozen baseline project
//! keeps every stock template and appends neutral ternary probes to
//! `machine_start_gcode`; each probe's failing expression sits in a dead
//! branch, while the selectors pick the live arm.

#[test]
fn ternary_inactive_branch_not_evaluated_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "template-ternary",
            &[
                "PlaceholderParser.cpp:2215-2221,851-871",
                "PlaceholderParser.cpp:1092-1094,1348-1355",
            ],
        ),
        "dead-branch",
        include_bytes!("../../../../../tests/parity/template-ternary/dead-branch.3mf"),
        "00d53c31b731281b2c9840960059e84a9626638190bb98b3d8b58d2f6153768c",
        (false, |reference, _| {
            let markers: Vec<&str> = reference
                .native
                .lines()
                .filter(|line| line.starts_with(";TERNARY_"))
                .collect();
            assert_eq!(
                markers,
                [
                    ";TERNARY_DEAD_DIVISION=7",
                    ";TERNARY_LIVE_SELECT=8",
                    ";TERNARY_NESTED=9",
                    ";TERNARY_DEAD_UNKNOWN=7",
                    ";TERNARY_DEAD_VECTOR=7",
                    ";TERNARY_DEAD_COMPARE=7",
                    ";TERNARY_DEAD_MODULO=7",
                    ";TERNARY_DEAD_FUNCTION=7",
                    ";TERNARY_DEAD_INDEX=7",
                    ";TERNARY_DEAD_SCALAR_INDEX=7",
                ],
                "Native must suppress inactive-branch evaluation"
            );
        }),
    );
}
