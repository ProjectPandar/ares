//! Stock Prusa MK4 tree-support base connect RED witness
//! (`FillBase.cpp:2203-2243` arc-cost evaluation / `:2493` sort).
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "mk4-tree-support-base-connect",
    &[
        "SupportCommon.cpp:1764-1816::generate_support_toolpaths base fill block",
        "FillRectilinear.cpp:3613-3634::FillSupportBase::fill_surface",
        "FillBase.cpp:2184-2201::evaluate_support_arch_cost",
        "Line.hpp:43-71::line_alg::distance_to_squared",
        "FillBase.cpp:2247-2650::Fill::connect_base_support",
    ],
);

#[test]
fn mk4_tree_support_base_connect_witness_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "witness",
        include_bytes!("../../../../tests/parity/mk4-tree-support-base-connect/witness.3mf"),
        "90650d0fdb5c17a2d84ad6f2c87e36ad0f3fc6a3f90645068c243e8738c10834",
        (false, |reference, _replay| {
            assert!(
                reference.native.contains(";TYPE:Support\n"),
                "witness must extrude support: {:?}",
                reference.root
            );
            // The proven first fork lives in the print_z 1.2 support layer.
            assert!(
                reference
                    .native
                    .contains(";Z:1.2\n;HEIGHT:0.2\n;BEFORE_LAYER_CHANGE"),
                "witness native must reach the print_z 1.2 support layer: {:?}",
                reference.root
            );
        }),
    );
}
