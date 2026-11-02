//! Standard beta gyroid period sampling on real low-density cylinder islands.
use crate::runner::native_stream_case::check;

#[test]
fn gyroid_low_density_boundary_waves_match_native_complete_output() {
    check(
        (
            "gyroid-standard-boundary-waves",
            &[
                "Fill/FillGyroid.cpp::make_one_period",
                "Fill/FillGyroid.cpp::make_wave",
                "Fill/FillGyroid.cpp::make_gyroid_waves",
                "Fill/FillGyroid.cpp::FillGyroid::_fill_surface_single",
            ],
        ),
        "paired-cylinders",
        include_bytes!(
            "../../../../../tests/parity/gyroid-standard-boundary-waves/paired-cylinders.3mf"
        ),
        "377c1bc060ac5b93d72d0b3b08af15d8a0f3fc3fb89c880b0a109db661fa5cd4",
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert!(native.native.contains(";TYPE:Sparse infill\n"));
                assert!(native.native.contains("; sparse_infill_pattern = gyroid\n"));
                assert!(native.native.contains("; sparse_infill_density = 1%\n"));
                assert!(native.native.contains("; gyroid_optimized = 0\n"));
                assert!(native.native.contains("; fill_multiline = 1\n"));
            }
        }),
    );
}
