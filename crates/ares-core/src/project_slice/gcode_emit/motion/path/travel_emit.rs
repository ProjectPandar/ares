use super::push_axis;

pub(super) fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

/// `GCodeWriter::travel_to_xy` (`GCodeWriter.cpp:873-890`): the writer subtracts its XY offset.
pub(super) fn xy_with_comment(
    output: &mut Vec<u8>,
    state: &super::super::EmitState,
    (x, y): (f64, f64),
    feedrate: f64,
    comment: &str,
) {
    output.extend_from_slice(b"G1 ");
    state.push_xy_words(output, (x, y), None);
    output.extend_from_slice(b" F");
    push_axis(output, feedrate);
    output.extend_from_slice(comment.as_bytes());
    output.push(b'\n');
}

/// Writer-domain XYZ from `GCodeWriter.cpp::travel_to_xyz` (`Vec3d`), less the writer's XY
/// offset (`:1114`), represented by the existing double-coordinate `Point3d`.
pub(super) fn xyz_with_comment(
    output: &mut Vec<u8>,
    state: &mut super::super::EmitState,
    crate::Point3d { x, y, z }: crate::Point3d,
    feedrate: f64,
    comment: &str,
) {
    output.extend_from_slice(b"G1 ");
    state.push_xy_words(output, (x, y), Some(z));
    output.extend_from_slice(b" F");
    push_axis(output, feedrate);
    output.extend_from_slice(comment.as_bytes());
    output.push(b'\n');
    state.record_z(z);
}
