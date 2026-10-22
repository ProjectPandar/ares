//! Smooth Spiral XY words use GCodeLine::set's fixed-three-decimal formatting.

#[test]
fn spiral_smooth_fixed_xy_off_control() {
    crate::runner::native_stream_case::check(
        (
            "spiral-smooth-fixed-xy",
            &["GCode/SpiralVase.cpp:182-183", "GCodeReader.cpp:323-326"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/spiral-smooth-fixed-xy/control.3mf"),
        "61e8e63712918c1c167d4276eb73cbd2881d9a76054f42c3b9a91ac6c058be26",
        (false, |_, _| {}),
    );
}

#[test]
fn spiral_smooth_fixed_xy_on_positive() {
    crate::runner::native_stream_case::check(
        (
            "spiral-smooth-fixed-xy",
            &["GCode/SpiralVase.cpp:182-183", "GCodeReader.cpp:323-326"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/spiral-smooth-fixed-xy/positive.3mf"),
        "a48e157624f34a7e7425e95dddafaeb7676692dc23674a03eac41a09efb2356e",
        (false, |_, _| {}),
    );
}
