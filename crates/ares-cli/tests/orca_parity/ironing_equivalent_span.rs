//! Equivalent ironing-region spans follow Layer::make_ironing.

#[test]
fn equal_ironing_parameter_span_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "ironing-equivalent-span",
            &["Fill/Fill.cpp:1507-1700::Layer::make_ironing/IroningParams"],
        ),
        "equal-angle-target",
        include_bytes!("../../../../tests/parity/ironing-equivalent-span/equal-angle-target.3mf"),
        "08ff42b871ae8cca22dfb529a059a5e5f2a2a633b73640dac87735b65d9679ab",
        false,
    );
}

#[test]
fn distinct_ironing_angle_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "ironing-equivalent-span",
            &["Fill/Fill.cpp:1507-1700::Layer::make_ironing/IroningParams"],
        ),
        "distinct-angle-control",
        include_bytes!(
            "../../../../tests/parity/ironing-equivalent-span/distinct-angle-control.3mf"
        ),
        "e9109b5d0a2e959c154133f7a3d63d94bdc5e9c952b0bdf6fc7c65ff2f7de8cb",
        false,
    );
}
