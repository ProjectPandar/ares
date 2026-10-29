//! `{elsif(...)}` keyword boundary (`PlaceholderParser.cpp:2110,2178-2186`).
//! The X1C filament-start template exercises that spelling with selected
//! Textured PEI bed vectors. The frozen baseline project differs only in
//! `curr_bed_type`; it does not replace the upstream template or temperatures.

#[test]
fn textured_pei_elsif_without_space_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "template-elsif",
            &[
                "PlaceholderParser.cpp:2110,2178-2186",
                "GCode.cpp:3023-3033,3043,3144-3150",
                "PrintConfig.hpp:466-510",
            ],
        ),
        "textured-pei",
        include_bytes!("../../../../../tests/parity/template-elsif/textured-pei.3mf"),
        "07e09729ff8cb51ea6d1bba27045dcd5f475be7f66edeed7593b5c3027bf9737",
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == "M140 S55 ;set bed temp")
            );
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == "M190 S55 ;wait for bed temp")
            );
            assert_eq!(
                reference
                    .native
                    .lines()
                    .filter(|line| *line == "M106 P3 S150")
                    .count(),
                1,
                "Native must select the stock filament-start elsif arm"
            );
        }),
    );
}
