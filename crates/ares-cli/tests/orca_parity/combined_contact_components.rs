//! Print::_make_skirt contact-connected brim components through public streams.

#[test]
fn near_combined_target() {
    check(
        "near-combined",
        include_bytes!("../../../../tests/parity/combined-contact-components/near-combined.3mf"),
        "0ebfc787f6cbaa1b2cc170964e0a96d6b72cbb7a2e8e0052b8a35ce992b1a801",
    );
}

#[test]
fn middle_combined_target() {
    check(
        "middle-combined",
        include_bytes!("../../../../tests/parity/combined-contact-components/middle-combined.3mf"),
        "c6ef1d191f8a70d63970483109e64fb3c303348cecb8fcf41892ab57c69758ad",
    );
}

#[test]
fn far_combined_target() {
    check(
        "far-combined",
        include_bytes!("../../../../tests/parity/combined-contact-components/far-combined.3mf"),
        "6180b58d4a30c4682af41cce20191078f92570b21a3c7fe2718af30974698393",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "combined-contact",
            &[
                "Print.cpp:2898-2990::Print::_make_skirt::make_group_brims",
                "GCode.cpp:4490-4537::generate_object_brim",
                "Brim.cpp:884-886::makeBrimInfillFromPlateCoordinates",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
