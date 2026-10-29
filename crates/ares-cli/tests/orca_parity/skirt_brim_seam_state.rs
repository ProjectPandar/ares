//! Pre-region skirt/brim reads inherit the previously applied region seam gap.

#[test]
fn previous_instance_region_seam_gap_target() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/skirt-brim-seam-state/target.3mf"),
        "76913b04187051e9837818e960491feffec96978d8b5f65a389378a4758ae850",
    );
}

#[test]
fn uniform_region_seam_gap_control() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/skirt-brim-seam-state/control.3mf"),
        "e4fb7cd74beb4add124b0691e7c0713ebfaad3a5571cff6a0e909a56a44318ba",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "skirt-brim-seam-state",
            &[
                "GCode.cpp:2533-2534,4593-4594,5334-5342::region config lifetime",
                "GCode.cpp:6136,6163::region config apply",
                "GCode.cpp:5793-5804::extrude_loop seam gap",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
