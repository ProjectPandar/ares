use super::super::native_stream_case;

macro_rules! complete_case {
    ($name:ident, $fixture:literal, $sha:literal) => {
        #[test]
        fn $name() {
            native_stream_case::check(
                (
                    "multiline-preoffset-crop",
                    &[
                        "FillRectilinear.cpp:3586-3601 FillStars dispatch",
                        "FillRectilinear.cpp:3433-3437 centerline crop before multiline_fill",
                        "FillBase.cpp:2947-3000 multiline_fill",
                    ],
                ),
                $fixture,
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/parity/multiline-preoffset-crop/",
                    $fixture,
                    ".3mf"
                )),
                $sha,
                (false, |_, _| {}),
            );
        }
    };
}

complete_case!(
    tri_hexagon_h2_matches_native_complete_output,
    "tri_hexagon_multiline_h2",
    "0c1af094e03982f1eb03d07bf7e2d78ecae5b73f6193061954111b49238b1c0b"
);
complete_case!(
    tri_hexagon_s2_matches_native_complete_output,
    "tri_hexagon_multiline_s2",
    "c1f3727c7e06d5320ada01f69902cd45f43ac62462ef5f1f792a301190a3d7e6"
);
complete_case!(
    rectilinear_named_s2_matches_native_complete_output,
    "rectilinear_multiline_s2",
    "4703dd0a443e0c62db7cc6c0c3446d882121888bcb7ac5e1c8d18a1030b959d0"
);
complete_case!(
    triangles_named_s2_matches_native_complete_output,
    "triangles_mode1_s2",
    "2f8a8bdde7d33057156e41d8ee90338f46fac75bda35ed01e67e03f657c5fad4"
);
