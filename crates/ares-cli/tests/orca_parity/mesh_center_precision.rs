//! Fresh ModelVolume bbox-center precision through Native/API/CLI streams.

#[test]
fn stepped_height54_target() {
    check(
        "step5.4-beam1",
        include_bytes!("../../../../tests/parity/mesh-center-precision/step5.4-beam1.3mf"),
        "114204a6de7dcaf101c5ea2ce525746f0ece22440ba3d680475b48b9c1f388af",
        true,
    );
}

#[test]
fn stepped_beam0_control() {
    check(
        "step5.4-beam0",
        include_bytes!("../../../../tests/parity/mesh-center-precision/step5.4-beam0.3mf"),
        "6ec7579e5b2e5887085505791d264bccec68885c5c16da70a4aee91bee0fdca0",
        true,
    );
}

#[test]
fn stepped_height94_control() {
    check(
        "step9.4-beam1",
        include_bytes!("../../../../tests/parity/mesh-center-precision/step9.4-beam1.3mf"),
        "e4903ade421dab2970cd8110238859190cc332dfe46c4843040bfb790c11162a",
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

fn check(label: &str, project: &[u8], sha: &str, two_tools: bool) {
    crate::runner::native_stream_case::check(
        (
            "mesh-center-precision",
            &[
                "TriangleMesh.cpp:bounding_box:431-437",
                "BoundingBox.cpp:center:231-237",
                "Model.cpp:center_geometry_after_creation:2600-2617",
            ],
        ),
        label,
        project,
        sha,
        (two_tools, |_, _| {}),
    );
}
