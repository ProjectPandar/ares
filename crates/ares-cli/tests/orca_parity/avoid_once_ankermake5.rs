//! `disable_once` arm/consumption ordering against `GCode.cpp:4448-4450`
//! (arm at skirt end), `:7429-7431` (`reset_once_modifiers` before the
//! retraction branch) and `:7444-7452` (post-wipe re-plan with the flag
//! already clear). See
//! `docs/architecture/ard-avoid-once-arm-consumption.md`.

#[test]
fn ankermake5_first_object_travel_detour() {
    crate::runner::native_stream_case::check(
        (
            "avoid-once-ankermake5",
            &[
                "GCode.cpp:generate_skirt:4413/:4448-4450",
                "GCode.cpp:travel_to:7415-7420/:7429-7431/:7444-7452",
                "GCode/AvoidCrossingPerimeters.hpp:23-25",
            ],
        ),
        "ankermake-m5",
        include_bytes!("../../../../tests/parity/avoid-once-ankermake5/ankermake-m5.3mf"),
        "c8e2c11d3e37ea0ec5f504dc0546f4011937834fb6227779f04bde0248f85929",
        (false, |_, _| {}),
    );
}
