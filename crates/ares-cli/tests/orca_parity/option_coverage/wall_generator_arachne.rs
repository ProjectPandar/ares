//! `Layer.cpp:38-67::make_slices` chains physical-layer `lslices` from
//! a local copy without reordering the region-owned surfaces consumed by
//! `PerimeterGenerator.cpp:2128::process_arachne`. Mutating that input order
//! changes the layer 6/7 fill-domain path order, then the layer 3 vertical-shell
//! projection and the variable-width concentric fill's observable E values.

use crate::runner::native_stream_case::check;

#[test]
fn orca_parity_wall_generator_arachne_matches_complete_output() {
    // This frozen KSR project needs the previously qualified seeded oracle:
    // uninitialized `; printing object` IDs make official double replay differ.
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    check(
        (
            "wall-generator-arachne",
            &[
                "Layer.cpp:38",
                "Layer.cpp:67",
                "PerimeterGenerator.cpp:2128",
            ],
        ),
        "x1c-wall-generator-arachne",
        include_bytes!(
            "../../../../../tests/parity/wall-generator-arachne/x1c-wall-generator-arachne.3mf"
        ),
        "c93f999d6ac47509ecf1f6a7c194df49bdba5a87099c261ba51e28d6200bcc98",
        (false, |_, _| {}),
    );
}
