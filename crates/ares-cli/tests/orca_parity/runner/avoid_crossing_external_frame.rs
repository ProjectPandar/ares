//! `GCode.cpp:10595-10617` at OrcaSlicer 2.5.0
//! `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`: when wiping changes
//! `last_pos()`, replan perimeter avoidance from the new start independently
//! of the initial writer-position condition. Preserve the external one-shot
//! modifier for that second call, then consume it. This unchanged two-object
//! X1 Carbon project compares the complete Native, API, and CLI streams.

#[test]
fn x1c_reduce_detour0_object_switch_travel() {
    crate::runner::native_stream_case::check(
        (
            "avoid-crossing-external-frame",
            &[
                "GCode.cpp:10600-10606:post-retraction-last-pos-and-direct-travel",
                "GCode.cpp:10606-10616:second-perimeter-planning-and-external-once",
                "AvoidCrossingPerimeters.cpp:1255-1325:travel_to",
            ],
        ),
        "x1c-detour0",
        include_bytes!("../../../../../tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf"),
        "f44f09c6b26391787d5877cfafd094bb50cb452d727298d46cd6ab5d45ffa04e",
        (false, |_, _| {}),
    );
}
