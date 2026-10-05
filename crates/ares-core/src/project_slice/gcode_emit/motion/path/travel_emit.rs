use super::super::format::z as format_z;
use super::format_axis;

pub(super) fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

pub(super) fn xy_with_comment(output: &mut Vec<u8>, x: f64, y: f64, feedrate: f64, comment: &str) {
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} F{}{comment}\n",
            format_axis(x),
            format_axis(y),
            format_axis(feedrate)
        )
        .as_bytes(),
    );
}

/// Writer-domain XYZ from `GCodeWriter.cpp::travel_to_xyz` (`Vec3d`),
/// represented by the existing double-coordinate `Point3d`.
pub(super) fn xyz_with_comment(
    output: &mut Vec<u8>,
    state: &mut super::super::EmitState,
    crate::Point3d { x, y, z }: crate::Point3d,
    feedrate: f64,
    comment: &str,
) {
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{} F{}{comment}\n",
            format_axis(x),
            format_axis(y),
            format_z(z),
            format_axis(feedrate)
        )
        .as_bytes(),
    );
    state.record_z(z);
}
