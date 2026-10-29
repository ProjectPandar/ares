//! `Config.cpp:690-737`, `PrintConfig.cpp:3224-3251` preserve raw f64
//! acceleration references before percentage resolution. `GCode.cpp:6414-6439`
//! converts only the selected role to an unsigned acceleration.
//! With outer1234.5 and bridge50%, Native emits617, not618; the changed
//! whole-job estimate first manifests as an earlier M73 P22 R10.

#[test]
fn voron_fractional_outer_acceleration_percentage_reference() {
    crate::runner::native_stream_case::check(
        (
            "acceleration-raw-f64",
            &[
                "Config.cpp:690-737:get_abs_value-raw-reference-chain",
                "PrintConfig.cpp:3224-3251:bridge-outer-and-infill-default-ratio-over",
                "GCode.cpp:6414-6439:resolve-role-before-floor-plus-half",
            ],
        ),
        "voron-outer1234-5",
        include_bytes!("../../../../../tests/parity/acceleration-raw-f64/voron-outer1234_5.3mf"),
        "cac9d740501e61bc2e32d149ad554e9289f98c28d04c65ee0e73aea726a55c58",
        (false, |_, _| {}),
    );
}
