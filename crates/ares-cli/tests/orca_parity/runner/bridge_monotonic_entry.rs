use super::super::native_stream_case;

macro_rules! bridge_case {
    ($name:ident, $fixture:literal, $sha:literal) => {
        #[test]
        fn $name() {
            native_stream_case::check(
                (
                    "bridge-monotonic-entry",
                    &[
                        "PrintObject.cpp:4073-4107 bridge-over-infill output",
                        "Fill.cpp:978-981 bridge pattern projection",
                        "FillRectilinear.cpp:3504-3512 FillMonotonic entry",
                    ],
                ),
                $fixture,
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/parity/plane-path-object-bounds/",
                    $fixture,
                    ".3mf"
                )),
                $sha,
                (false, |_, _| {}),
            );
        }
    };
}

bridge_case!(
    hilbert_uncompensated_bridge_matches_native_complete_output,
    "hilbert-compensation0",
    "50db1ed174c0914cbfbe0b36ca4d4b21e22d36acca3125e05b343c3db2af92f2"
);
bridge_case!(
    hilbert_compensated_bridge_matches_native_complete_output,
    "hilbert-compensation01",
    "2d59332f77beced29dddeff87eded5d8fb6d980042c8566cd46c36de630644b3"
);
bridge_case!(
    octagram_compensated_bridge_matches_native_complete_output,
    "octagram-compensation01",
    "12d4b5c3d98e26014e3166dfaf90a2dde8b57bb3a8e5120bd40393c3e2e7e970"
);
