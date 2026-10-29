//! Skirt fallback speed reads the object configuration active at emission.

#[test]
fn previous_object_support_speed_target() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/skirt-support-speed-state/target.3mf"),
        "8db2eaacfd6ad62ad1c0937f5fa2c37a7f2f88f08b8160545e07db5d9a1bf33d",
    );
}

#[test]
fn uniform_support_speed_control() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/skirt-support-speed-state/control.3mf"),
        "931449cc699bac934bfe399a59e9447a91d2ce28a58c8cf674113035c7e812ef",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "skirt-support-speed-state",
            &[
                "GCode.cpp:4434-4441::generate_skirt explicit support speed",
                "GCode.cpp:4593-4594,5341-5342::object configuration apply",
                "GCode.cpp:6598-6603::positive skirt speed override",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
