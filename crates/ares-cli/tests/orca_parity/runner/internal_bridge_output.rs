use super::super::native_stream_case;

macro_rules! bridge_case {
    ($name:ident, $fixture:literal, $sha:literal) => {
        #[test]
        fn $name() {
            native_stream_case::check(
                (
                    "internal-bridge-output",
                    &[
                        "PrintObject.cpp:3872-3886 whole bridge components",
                        "PrintObject.cpp:4073-4107 internal bridge surface rewrite",
                        "Fill/Fill.cpp:940-1080,1507-1513 bridge projection",
                    ],
                ),
                $fixture,
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/parity/internal-bridge-output/",
                    $fixture,
                    ".3mf"
                )),
                $sha,
                (false, |reference, replay| {
                    for native in [reference, replay] {
                        assert!(
                            native.native.contains(";TYPE:Internal Bridge\n"),
                            "Native target must execute internal bridge extrusion: {:?}",
                            native.root
                        );
                    }
                }),
            );
        }
    };
}

bridge_case!(
    honeycomb_count2_matches_native_complete_output,
    "honeycomb-count2",
    "e43bf5ac79bcb39eae1cfeb0f2ab0d93b97627fb83368473bb1760f49e16a07c"
);
bridge_case!(
    triangles_count1_matches_native_complete_output,
    "triangles-count1",
    "58efc7d45f6ae6df74c711a7a7d5caa80f2cbf8e01ab0045039d0b3b9c0f8049"
);
bridge_case!(
    triangles_count2_matches_native_complete_output,
    "triangles-count2",
    "29cb5b6f383537a1c5da393bba66b3f5e61b83b01df3b1e52cdde2561bcc0c5e"
);
