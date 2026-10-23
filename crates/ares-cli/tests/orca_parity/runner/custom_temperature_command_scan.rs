//! GCode.cpp:528-589 custom-temperature recognition before machine start output.

use crate::runner::{native_api_cli::NativeCase, native_stream_case};

#[test]
fn compact_bed_commands_suppress_generated_bed_wait() {
    check(
        "compact-bed",
        include_bytes!(
            "../../../../../tests/parity/custom-temperature-command-scan/compact-bed.3mf"
        ),
        "e78d5230a495fd486ee289496d1fd7ccd6f2e903a33efa3f3afb423a415890bf",
        |native, _| {
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M140S60"))
            );
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M190S60"))
            );
            assert_no_generated_heating(native);
        },
    );
}

#[test]
fn compact_nozzle_commands_suppress_generated_heating() {
    check(
        "compact-nozzle",
        include_bytes!(
            "../../../../../tests/parity/custom-temperature-command-scan/compact-nozzle.3mf"
        ),
        "9a41b0b7d33f19f322f45214dff1f182c4bd1479ed0419ab5e7da10f02280fed",
        |native, _| {
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M104S150"))
            );
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M109S220"))
            );
            assert_no_generated_heating(native);
        },
    );
}

#[test]
fn spaced_commands_control_stays_byte_equal() {
    check(
        "spaced-control",
        include_bytes!(
            "../../../../../tests/parity/custom-temperature-command-scan/spaced-control.3mf"
        ),
        "a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae",
        |native, _| {
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M140 S60"))
            );
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line.starts_with("M109 S220"))
            );
            assert_no_generated_heating(native);
        },
    );
}

#[test]
fn commented_commands_do_not_suppress_generated_heating() {
    check(
        "comment-control",
        include_bytes!(
            "../../../../../tests/parity/custom-temperature-command-scan/comment-control.3mf"
        ),
        "44a5499fc36ebc53b457beccc7b36598d102875e4a6c231e64791464854e6b82",
        |native, _| {
            assert!(
                native
                    .native
                    .contains("; M140S60 M190S60 M104S220 M109S220\n")
            );
            let prefix = native.native.split_once(";TYPE:Custom\n").unwrap().0;
            assert!(prefix.lines().any(|line| {
                line == "M190 S60 ; set bed temperature and wait for it to be reached"
            }));
            assert!(
                prefix
                    .lines()
                    .any(|line| line == "M104 S220 ; set nozzle temperature")
            );
        },
    );
}

fn assert_no_generated_heating(native: &NativeCase) {
    let prefix = native.native.split_once(";TYPE:Custom\n").unwrap().0;
    assert!(!prefix.lines().any(|line| {
        ["M104", "M109", "M140", "M190"]
            .iter()
            .any(|command| line.starts_with(command))
    }));
}

fn check(label: &str, input: &[u8], sha: &str, qualify: impl Fn(&NativeCase, &NativeCase)) {
    native_stream_case::check(
        (
            "custom-temperature-scan",
            &[
                "GCode.cpp:528-589::custom_gcode_sets_temperature",
                "GCode.cpp:4025-4054::_print_first_layer_bed_temperature",
                "GCode.cpp:4058-4090::_print_first_layer_extruder_temperatures",
            ],
        ),
        label,
        input,
        sha,
        (false, qualify),
    );
}
