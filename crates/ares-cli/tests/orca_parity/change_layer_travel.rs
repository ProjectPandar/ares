//! GCode.cpp:9224-9246: after a layer change, `_extrude` travels to the first
//! point even when the nozzle already stands on it.

#[test]
fn change_layer_travel_restates_the_first_point_like_native() {
    crate::runner::native_stream_case::check(
        (
            "change-layer-travel",
            &["GCode.cpp:9224-9246", "GCode.cpp:10488-10504"],
        ),
        "octagon-seam0",
        include_bytes!("../../../../tests/parity/change-layer-travel/octagon-seam0.3mf"),
        "17dcf83bdf192c720c83b3a91a506ad82222f203af874b3bd451221d34003716",
        (false, |_, _| {}),
    );
}
