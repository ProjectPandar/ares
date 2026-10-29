//! `Generator.cpp:166` offsets the complete winding-aware Polygons collection.

#[test]
fn orca_parity_lightning_initial_overhang_polygons_a1() {
    crate::runner::native_stream_case::check(
        (
            "lightning-initial-overhang-polygons",
            &[
                "Fill/Lightning/Generator.cpp:151-170",
                "ClipperUtils.cpp:265-300,366-408",
            ],
        ),
        "a1-lightning15",
        include_bytes!(
            "../../../../tests/parity/lightning-initial-overhang-polygons/a1-lightning15.3mf"
        ),
        "fbba9c87450ad465300b3d522912068e9625395faf3feed937eaa4267f54c067",
        (false, |_, _| {}),
    );
}
