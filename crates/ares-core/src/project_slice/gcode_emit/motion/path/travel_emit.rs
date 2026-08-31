use super::super::format::z as format_z;
use super::format_axis;

pub(super) fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

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

pub(super) fn xy_without_feed(output: &mut Vec<u8>, x: f64, y: f64) {
    output.extend_from_slice(format!("G1 X{} Y{}\n", format_axis(x), format_axis(y)).as_bytes());
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
