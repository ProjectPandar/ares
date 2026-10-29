//! Loop-end inward displacement through public Native/API/CLI streams.

#[test]
fn orthogonal_original_segment() {
    check(
        "orthogonal-grid-beam1",
        include_bytes!("../../../../tests/parity/inward-displacement/orthogonal-grid-beam1.3mf"),
        "afed0f4f4417916d2963363754bc45ef1300f6bbc2d6dfbcc9d52cf751a0c312",
        true,
    );
}

#[test]
fn default_beam_control() {
    check(
        "default-beam1",
        include_bytes!("../../../../tests/parity/interlocking-base/two-extruder-beam1.3mf"),
        "04c7d2a58b7b21909b867542faad149a6b9d0b1bb976eab3fec57b7b60ff082c",
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
            "inward-displacement",
            &[
                "GCode.cpp:extrude_loop:5996-6023",
                "Point.cpp:ccw_angle:81-89",
            ],
        ),
        label,
        project,
        expected_sha,
        (assert_two_tools, |_, _| {}),
    );
}
