//! Direct registered coInts placeholder identity through public streams.

#[test]
fn direct_coints_control_matches_native() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/direct-coints-placeholder/control.3mf"),
        "7a589f23fe9bb4c1bc129126e780c77eaaafa9fadde7be9a472bf9ea47f792e4",
    );
}

#[test]
fn direct_coints_signed_target_matches_native() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/direct-coints-placeholder/target.3mf"),
        "ed81e291104afdc74f93772866145664b27aacabffe03752b86081310f4f0f4f",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "direct-coints-placeholder",
            &[
                "PrintConfig.cpp:2917-2923 temperature_vitrification coInts",
                "Config.hpp:995-1006,1070-1077 ConfigOptionInts serialization",
                "PlaceholderParser.cpp:932-945,1212 coInts expansion",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
