//! Machine-start overall_chamber_temperature retains upstream integer identity.

#[test]
fn chamber_even_control_matches_native() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/overall-chamber-placeholder/control.3mf"),
        "f14985d07827445123e4e42ed68582e94c622bba651f12c472ab9ec7d87cfcc7",
    );
}

#[test]
fn chamber_odd_target_matches_native() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/overall-chamber-placeholder/target.3mf"),
        "ddb1daa8c94b29eeaf41e5f92f33768a69465da99dad9abacf926f9f1b436e31",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "overall-chamber-placeholder",
            &[
                "GCode.cpp:3009-3012,3037 ConfigOptionInt(max_chamber_temp)",
                "Config.hpp:954-972 ConfigOptionInt",
                "PlaceholderParser.cpp:494-509,1040-1041 integer division/type",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
