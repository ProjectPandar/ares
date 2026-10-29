//! Fill.cpp:937,941,1136 template identity vs. FillAlignedRectilinear layer angle.

use super::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "aligned-template-identity",
    &[
        "libslic3r/Fill/Fill.cpp:288,316,937,941,1136",
        "libslic3r/Fill/FillRectilinear.hpp:38-46",
        "libslic3r/Fill/FillBase.cpp:308-312",
    ],
);

#[test]
fn aligned_same_angle_modifier_template_matches_native_complete_output() {
    check(
        SOURCE,
        "aligned-local45",
        include_bytes!("../../../../../tests/parity/aligned-template-identity/aligned-local45.3mf"),
        "779f7758a8ca330b2cbd2cf0688b1c7a154bbd44297f36c3af977c9f433956f9",
        (false, |_, _| {}),
    );
}

#[test]
fn aligned_uniform_explicit_template_control_preserves_complete_output() {
    check(
        SOURCE,
        "aligned-global45-local45",
        include_bytes!(
            "../../../../../tests/parity/aligned-template-identity/aligned-global45-local45.3mf"
        ),
        "14987398e25c5ba9483311f74a2c86ce39f6ae5db236583831abb15cdcba3954",
        (false, |_, _| {}),
    );
}

#[test]
fn aligned_different_angle_modifier_control_preserves_complete_output() {
    check(
        SOURCE,
        "aligned-local90",
        include_bytes!("../../../../../tests/parity/aligned-template-identity/aligned-local90.3mf"),
        "19ee9aa23dfe578d4542f8e785fc27f0048fbb2bd930eee783da70bd2b737630",
        (false, |_, _| {}),
    );
}
