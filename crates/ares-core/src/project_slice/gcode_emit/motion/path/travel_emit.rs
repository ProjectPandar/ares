use super::super::format::z as format_z;
use super::format_axis;

pub(super) fn xy(output: &mut Vec<u8>, x: f64, y: f64, feedrate: f64) {
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} F{}\n",
            format_axis(x),
            format_axis(y),
            format_axis(feedrate)
        )
        .as_bytes(),
    );
}

pub(super) fn xyz(output: &mut Vec<u8>, x: f64, y: f64, z: f64, feedrate: f64) {
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{} F{}\n",
            format_axis(x),
            format_axis(y),
            format_z(z),
            format_axis(feedrate)
        )
        .as_bytes(),
    );
}
