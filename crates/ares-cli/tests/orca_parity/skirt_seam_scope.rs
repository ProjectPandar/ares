//! Initial combined skirt sees default region state, not object region overrides.

#[test]
fn object_seam_gap_override_target() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/skirt-seam-scope/target.3mf"),
        "fe99c85d5beb6585633f12ddd518aaa37bc88b012da9b6819dd6bcfb6a9615c6",
    );
}

#[test]
fn default_seam_gap_control() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/skirt-seam-scope/control.3mf"),
        "d3f6e170ab2162df20575cd39ec9f289df2a899ce2d13ec10040d01666156b12",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "skirt-seam-scope",
            &[
                "GCode.cpp:2533-2534,4593-4594,5243-5251::process_layer config scope",
                "GCode.cpp:4388-4451::generate_skirt",
                "GCode.cpp:5793-5804::extrude_loop seam gap",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
