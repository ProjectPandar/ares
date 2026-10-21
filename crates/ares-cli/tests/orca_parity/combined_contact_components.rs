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

#[test]
fn near_uncombined_control() {
    check(
        "near-uncombined",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/near-uncombined.3mf"),
        "c73b006dee07894432046ef2b16724824e788bd2e75ab219338636870638ad56",
    );
}

#[test]
fn far_uncombined_control() {
    check(
        "far-uncombined",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/far-uncombined.3mf"),
        "c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6",
    );
}

#[test]
fn middle_uncombined_control() {
    check(
        "middle-uncombined",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/middle-uncombined.3mf"),
        "c44fb386d4c0bff36e23a643c6e112182552497f4dc7f83605465f8af4e1dbf2",
    );
}

#[test]
fn near_no_brim_control() {
    check(
        "near-no-brim",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/near-no-brim.3mf"),
        "6b2e46a061dee196d1a2a837e1cbd0c2502001bee06eca0ac65732f81dac382f",
    );
}

#[test]
fn far_no_brim_control() {
    check(
        "far-no-brim",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/far-no-brim.3mf"),
        "71e2690d381d7698a94ce5c6d53eb2da5359fe00de18029c26c10a8b707a1695",
    );
}

#[test]
fn middle_no_brim_control() {
    check(
        "middle-no-brim",
        include_bytes!("../../../../tests/parity/ordered-brim-owner/middle-no-brim.3mf"),
        "be1176b4cb398beffd797898ebed6844be8e665cb52d63160767c4a743aafde4",
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
        false,
    );
}
