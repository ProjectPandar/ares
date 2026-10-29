//! GCodeWriter::extrude_to_xy emits the role comment even when E is omitted.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "zero-extrusion-comment",
    &[
        "GCodeWriter.cpp:933-956",
        "GCode.cpp:2038,7031-7036",
        "PrintConfig.cpp:1366-1375",
    ],
);

#[test]
fn zero_extrusion_comment_normal_flow_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/zero-extrusion-comment/control.3mf"),
        "ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632",
        (false, |_, _| {}),
    );
}

#[test]
fn zero_extrusion_comment_zero_flow_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/zero-extrusion-comment/target.3mf"),
        "90ca1740ea4fece67054f2fdeb4202ed31f7fd43b521f9976aacf9647550631b",
        (false, |_, _| {}),
    );
}
