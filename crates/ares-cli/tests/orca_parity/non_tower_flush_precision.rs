use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "non-tower-flush-precision",
    &[
        "GCode.cpp:11151-11194::GCode::set_extruder wipe_volume",
        "GCode.cpp:11207,11327-11341::GCode::set_extruder flush lengths",
    ],
);

fn check_support_tool(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        UPSTREAM,
        label,
        project,
        sha,
        (true, |reference, _| {
            assert!(reference.native.contains("; FEATURE: Support\n"));
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == "G1 E0.0972428 F50")
            );
        }),
    );
}

#[test]
fn towerless_support_tool_flush_matches_native() {
    check_support_tool(
        "support",
        include_bytes!("../../../../tests/parity/non-tower-flush-precision/support.3mf"),
        "2bdce3bb5a53a5148ca3e7af9733af563f65a99669a6bf05d4c2c75cd4022b62",
    );
}

#[test]
fn towerless_support_interface_tool_flush_matches_native() {
    check_support_tool(
        "interface",
        include_bytes!("../../../../tests/parity/non-tower-flush-precision/interface.3mf"),
        "d3b456554fcdbd51530399867df7aa88d96e8d8b1c85b15b1ad1054d84d34c4c",
    );
}

#[test]
fn towerless_single_filament_support_matches_native_without_switching_tools() {
    native_stream_case::check(
        UPSTREAM,
        "single",
        include_bytes!("../../../../tests/parity/non-tower-flush-precision/single.3mf"),
        "8f77606483edd4fb6de7ee4f303db4affb6da5a79c93d380413bd0942fdac1f5",
        (false, |reference, _| {
            assert!(reference.native.contains("; FEATURE: Support\n"));
            assert!(!reference.native.lines().any(|line| line.starts_with("T1")));
        }),
    );
}
