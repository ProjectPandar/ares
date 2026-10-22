//! Initial numeric generator cursor through public Native/API/CLI streams.

#[test]
fn front_edge_offset_target_matches_native() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/front-edge-numeric-cursor/target.3mf"),
        "d70b49101439c4a040cad4bbba3baf728d0d7d39d97963e59718ed245a5087db",
    );
}

#[test]
fn front_edge_zero_offset_control_matches_native() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/front-edge-numeric-cursor/control.3mf"),
        "0f5ff07e6145155275ba82bbedfd608569117a3ee30a26309686efba2764f0c1",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "front-edge-numeric-cursor",
            &[
                "Point.hpp:277 Point3 numeric initialization",
                "GCode.hpp:197,224,397-399,607-608 numeric cursor/defined flag",
                "GCode.cpp:5659-5669 set_origin;5769-5773 extrude_loop",
                "GCode/SeamPlacer.cpp:1554-1558 spNearest preferred point",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
