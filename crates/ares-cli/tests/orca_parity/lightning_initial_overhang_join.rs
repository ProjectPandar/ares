//! Lightning initial-overhang offset uses the upstream default miter join.

#[path = "lightning_initial_overhang_polygons.rs"]
mod lightning_initial_overhang_polygons;

#[test]
fn lightning_initial_overhang_join_grid_control() {
    crate::runner::native_stream_case::check(
        (
            "lightning-initial-overhang-join",
            &[
                "Fill/Lightning/Generator.cpp:166",
                "ClipperUtils.hpp:19,26,339",
            ],
        ),
        "control",
        include_bytes!("../../../../tests/parity/lightning-initial-overhang-join/control.3mf"),
        "00ed2a57ebe7bc50fe19aa2d0ff977304c25421c93876eae4c8c4aecbb20718c",
        (false, |_, _| {}),
    );
}

#[test]
fn lightning_initial_overhang_join_lightning_positive() {
    crate::runner::native_stream_case::check(
        (
            "lightning-initial-overhang-join",
            &[
                "Fill/Lightning/Generator.cpp:166",
                "ClipperUtils.hpp:19,26,339",
            ],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/lightning-initial-overhang-join/positive.3mf"),
        "3622362104e0fb943cf8b4fb7277d168e8f6285633a5e21d10636e2f7c1472d1",
        (false, |_, _| {}),
    );
}
