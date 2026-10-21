//! Endpoint-contact simplification through public Native/API/CLI streams.

#[test]
fn beam_disabled_height025() {
    check(
        "height025-beam0",
        include_bytes!("../../../../tests/parity/endpoint-contact/height025-beam0.3mf"),
        "b1c6d193f0b990473cc857fdbde50a4ddfc7a739aff30bf3ce3810529a0e9529",
        true,
    );
}

#[test]
fn original_odd_kernel_residual() {
    check(
        "odd-kernel-beam1",
        include_bytes!("../../../../tests/parity/endpoint-contact/odd-kernel-beam1.3mf"),
        "180d44d516f560510bb19a6584e6f41cd092cb16788a6a4091d7a32c69d841b7",
        true,
    );
}

#[test]
fn single_volume_control() {
    check(
        "single",
        include_bytes!("../../../../tests/parity/interlocking-base/single.3mf"),
        "1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651",
        false,
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str, assert_two_tools: bool) {
    crate::runner::native_stream_case::check(
        (
            "endpoint-contact",
            &[
                "GCode/AvoidCrossingPerimeters.cpp:FirstIntersectionVisitor/simplify_travel",
                "Geometry.hpp:segments_intersect",
            ],
        ),
        label,
        project,
        expected_sha,
        (assert_two_tools, |_, _| {}),
    );
}
