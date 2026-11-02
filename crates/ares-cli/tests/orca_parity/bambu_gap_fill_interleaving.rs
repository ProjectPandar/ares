//! Beta `FillBase.cpp:232-302,353-356`: fixed-order gap interleaving.

use crate::runner::native_stream_case;

#[test]
fn monotonic_gap_interleaving_matches_native_complete_output() {
    native_stream_case::check(
        (
            "bambu-gap-fill-interleaving",
            &[
                "Fill/FillBase.cpp::insert_gap_fill:232-302;_create_gap_fill:353-356",
                "Fill/FillRectilinear.hpp::FillMonotonic:57-65",
            ],
        ),
        "velocity-limit",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bambu-gap-fill-interleaving/velocity-limit.3mf"
        )),
        "d8f74deff1274800b04f021ba812c5fb84c1629e45981653218963e2cc3a1e37",
        (false, |reference, _| {
            for marker in [
                "; internal_solid_infill_pattern = monotonic\n",
                "; gap_fill_target = everywhere\n",
                ";TYPE:Internal solid infill\n",
                ";TYPE:Gap infill\n",
            ] {
                assert!(reference.native.contains(marker));
            }
        }),
    );
}
