//! `GCode.cpp:10519-10531`: first-layer travel acceleration uses `on_first_layer`.

#[test]
fn elevated_support_uses_ordinary_travel_acceleration() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-acceleration-predicate",
            &[
                "GCode.cpp:10519-10531::travel_to",
                "GCode.hpp:991::on_first_layer",
                "Layer.hpp:152::bottom_z",
                "SupportCommon.cpp:1492-1540::support_layer_ids",
            ],
        ),
        "inset-roof-matched",
        include_bytes!(
            "../../../../tests/parity/first-layer-travel-acceleration-predicate/inset-roof-matched.3mf"
        ),
        "4a844749b2534d2fde66670ff4fcf6844c661f0c9d4d25520c06bd1dbcdf60fa",
        (false, |reference, _| {
            let elevated_support = reference
                .native
                .split(";LAYER_CHANGE\n")
                .find(|layer| layer.lines().any(|line| line.starts_with(";TYPE:Support")))
                .expect("project must emit support");
            assert!(elevated_support.starts_with(";Z:2.4\n"));
            assert!(elevated_support.lines().any(|line| line == "M204 S1400"));
            assert!(!elevated_support.lines().any(|line| line == "M204 S650"));
        }),
    );
}
