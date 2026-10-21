//! GCode.cpp:5769-5775: configured spiral bypasses perimeter SeamPlacer.

#[test]
fn spiral_placement_gate_nonspiral_control() {
    crate::runner::native_stream_case::check(
        ("spiral-placement-gate", &["GCode.cpp:5769-5775"]),
        "control",
        include_bytes!("../../../../tests/parity/spiral-placement-gate/control.3mf"),
        "89e9b81620b4b3d210f3ae894e6041f03d4638e40ce240d134614e9e866095e0",
        (false, |_, _| {}),
    );
}

#[test]
fn spiral_placement_gate_octagon_positive() {
    crate::runner::native_stream_case::check(
        ("spiral-placement-gate", &["GCode.cpp:5769-5775"]),
        "positive",
        include_bytes!("../../../../tests/parity/spiral-placement-gate/positive.3mf"),
        "aa361aa6fe7c0a5bb03a5eb8e891e01ffd04cba687d427d35db1f25fea24bed4",
        (false, |_, _| {}),
    );
}

#[test]
fn spiral_placement_gate_cw_rectangle_regression() {
    crate::runner::native_stream_case::check(
        ("spiral-placement-gate", &["GCode.cpp:5769-5775"]),
        "cw-rectangle",
        include_bytes!("../../../../tests/parity/spiral-placement-gate/cw-rectangle.3mf"),
        "94a5b7e51d536faae94e4900b17f4eadf92fb0dea4c7c3d3f2c53c7f4231a48a",
        (false, |_, _| {}),
    );
}
