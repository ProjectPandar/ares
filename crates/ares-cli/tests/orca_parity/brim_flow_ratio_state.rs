//! Brim flow uses the object configuration active at extrusion time.

#[test]
fn object_brim_ratio_target() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/brim-flow-ratio-state/target.3mf"),
        "726408c79aa4127f4bd3796c1b4277ca36c153f4e489722d5b45de89485f4d4f",
    );
}

#[test]
fn uniform_brim_ratio_control() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/brim-flow-ratio-state/control.3mf"),
        "534ea70d9a62b1e4e6d6787066941023a2fce314436d39306d12aeccaf16e913",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "brim-flow-ratio-state",
            &[
                "GCode.cpp:6479-6480::_extrude erBrim flow ratio",
                "GCode.cpp:4593-4594,5334-5342::object configuration apply",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
