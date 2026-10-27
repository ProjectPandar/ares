//! `GCode.cpp:7371-7375` converts resolved first-layer travel acceleration
//! to an unsigned local before its positive gate and final floor(+0.5).
//! `:7393-7394` keeps the ordinary travel acceleration in the raw domain.

#[test]
fn voron_fractional_initial_travel_acceleration() {
    crate::runner::native_stream_case::check(
        (
            "initial-travel-uint",
            &[
                "GCode.cpp:7371-7375:first-layer-unsigned-local-before-rounding",
                "GCode.cpp:7393-7394:ordinary-travel-raw-floor-plus-half",
                "GCode.cpp:7409-7410:klipper-travel-writer",
            ],
        ),
        "voron-travel1234-5",
        include_bytes!("../../../../../tests/parity/initial-travel-uint/voron-travel1234_5.3mf"),
        "2f62e42ce65235ceb7d2ce49b0a32638839f203b01c65baa125943f67f7700bc",
        (false, |_, _| {}),
    );
}
