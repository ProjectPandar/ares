//! `GCodeWriter` printable-area circle admission through public slicing streams.

#[test]
fn front_edge_circle_uses_native_normal_lift() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/front-edge-numeric-cursor/control.3mf"),
        "0f5ff07e6145155275ba82bbedfd608569117a3ee30a26309686efba2764f0c1",
        "G1 Z1.2 F30000\nG1 X25.21 Y1.29 Z1.2\nG1 Z.8\n",
    );
}

#[test]
fn offset_front_edge_circle_uses_native_normal_lift() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/front-edge-numeric-cursor/target.3mf"),
        "d70b49101439c4a040cad4bbba3baf728d0d7d39d97963e59718ed245a5087db",
        "G1 Z1 F30000\nG1 X34.79 Y-.71 Z1\nG1 Z.6\n",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str, normal_lift: &str) {
    crate::runner::native_stream_case::check(
        (
            "spiral-lift-printable-area",
            &[
                "GCodeWriter.cpp:181-225 apply_print_config printable-area capture",
                "GCodeWriter.cpp:229-260 active_printable_area/spiral_lift_fits_printable_area",
                "GCodeWriter.cpp:950-965 eager_lift circle admission",
                "GCodeWriter.cpp:1023-1036 travel_to_xyz circle admission",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |reference, _| {
            assert!(
                reference.native.contains(normal_lift),
                "Native must replace the out-of-area circle with a normal lift: {:?}",
                reference.root
            );
        }),
    );
}
