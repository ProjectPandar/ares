//! Print::_make_skirt per-extruder minimum filament budget, public streams.

#[test]
fn two_tools_positive_minimum_target() {
    check(
        "tools2-minimum10",
        include_bytes!("../../../../tests/parity/skirt-minlength-multitool/tools2-minimum10.3mf"),
        "cd9460b1c11e7dff39c5f1f4ffb60e9558489efd1edf7e3abb90ea3ff2a7caba",
        true,
    );
}

#[test]
fn two_tools_zero_minimum_control() {
    check(
        "tools2-minimum0",
        include_bytes!("../../../../tests/parity/skirt-minlength-multitool/tools2-minimum0.3mf"),
        "db9233ceef4f11ec285030264edd2f9231d504a0ec2c5d2e9c37af8f813e61d8",
        true,
    );
}

#[test]
fn one_tool_positive_minimum_control() {
    check(
        "tools1-minimum10",
        include_bytes!("../../../../tests/parity/skirt-minlength-multitool/tools1-minimum10.3mf"),
        "f621a44b830b18c21139adb91926560dc938e14b7a5410330414669605eaeb3b",
        false,
    );
}

#[test]
fn one_tool_zero_minimum_control() {
    check(
        "tools1-minimum0",
        include_bytes!("../../../../tests/parity/skirt-minlength-multitool/tools1-minimum0.3mf"),
        "5d40f7b3764ae88c6ee5dd730b4f46841203eed5ebe62934d5464bc76175dbe8",
        false,
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str, assert_two_tools: bool) {
    crate::runner::native_stream_case::check(
        (
            "skirt-minlength-multitool",
            &[
                "Print.cpp:2710-2767::Print::_make_skirt::append_skirt_loops_for_hull",
                "GCode.cpp:4262-4278::Skirt::skirt_loops_per_extruder_all_printing",
            ],
        ),
        label,
        project,
        expected_sha,
        (assert_two_tools, |_, _| {}),
    );
}
