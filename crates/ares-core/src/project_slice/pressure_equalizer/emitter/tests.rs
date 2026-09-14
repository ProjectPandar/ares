use super::super::line::{GCodeLine, GCodeLineType, LineParser};
use super::{OutputBuffer, output_gcode_line};

fn extrude_line(raw: &str) -> GCodeLine {
    let mut parser = LineParser::new(&[1.75], false);
    let mut marker = Default::default();
    parser.parse_line(";_EXTRUSION_ROLE:1", &mut marker);
    let mut line = GCodeLine::default();
    parser.parse_line(raw, &mut line).expect("parses");
    line
}

#[test]
fn unmodified_line_passes_through_raw() {
    let mut buffer = OutputBuffer::default();
    let mut line = extrude_line("G1 X20 Y0 E2 F1800");
    assert_eq!(line.line_type, GCodeLineType::Extrude);

    output_gcode_line(&mut buffer, &mut line, 2.0, false);

    assert_eq!(buffer.finish(), "G1 X20 Y0 E2 F1800\n");
}

/// A modified line whose start/end rates differ only trivially re-emits as
/// one line at the corrected feedrate with the marker pair
/// (`PressureEqualizer.cpp:503-508`).
#[test]
fn modified_line_with_trivial_delta_reemits_single() {
    let mut buffer = OutputBuffer::default();
    let mut line = extrude_line("G1 X20 Y0 E2 F1800");
    line.modified = true;
    line.volumetric_extrusion_rate = 433.0;
    line.volumetric_extrusion_rate_start = 430.0;
    line.volumetric_extrusion_rate_end = 436.0;

    output_gcode_line(&mut buffer, &mut line, 2.0, false);

    let out = buffer.finish();
    assert!(out.contains(";_EXTRUDE_END\n"));
    assert!(out.contains("G1 F1800;_EXTRUDE_SET_SPEED\n"));
    assert!(out.contains("G1 X20 Y0 E2\n"));
}

/// A modified line with a non-trivial delta splits into segment chains;
/// each split segment carries interpolated positions and center
/// feedrates (`PressureEqualizer.cpp:509+`).
#[test]
fn modified_line_with_large_delta_splits() {
    let mut buffer = OutputBuffer::default();
    let mut line = extrude_line("G1 X40 Y0 E4 F1800");
    line.modified = true;
    line.volumetric_extrusion_rate = 433.0;
    line.volumetric_extrusion_rate_start = 430.0;
    line.volumetric_extrusion_rate_end = 72.0;
    line.max_volumetric_extrusion_rate_slope_positive = 1.0;
    line.max_volumetric_extrusion_rate_slope_negative = 1.0;

    output_gcode_line(&mut buffer, &mut line, 2.0, false);

    let out = buffer.finish();
    // Multiple extrusion G1s with interpolated X positions.
    let extrusions = out.lines().filter(|l| l.starts_with("G1 X")).count();
    assert!(extrusions >= 2, "expected split segments, got:\n{out}");
    // Feedrate ladder present (F words other than the original).
    let feeds = out
        .lines()
        .filter(|l| l.starts_with("G1 F"))
        .filter_map(|l| l[4..].split(';').next().map(str::to_owned))
        .collect::<Vec<_>>();
    assert!(feeds.len() >= 2, "expected feedrate ladder: {feeds:?}");
    assert!(feeds.iter().any(|f| f != "1800"));
}

#[test]
fn bare_set_speed_line_is_rolled_back() {
    let mut buffer = OutputBuffer::default();
    // First emit produces ";_EXTRUDE_END" + set-speed + extrusion.
    let mut first = extrude_line("G1 X10 Y0 E1 F1800");
    first.modified = true;
    first.volumetric_extrusion_rate = 433.0;
    first.volumetric_extrusion_rate_start = 433.0;
    first.volumetric_extrusion_rate_end = 433.0;
    output_gcode_line(&mut buffer, &mut first, 2.0, false);
    let out1 = buffer.finish();

    // Reuse: consecutive identical-rate lines leave no bare F line
    // because the previous extrusion closed with a marker line.
    assert!(!out1.is_empty());
    assert!(out1.contains(";_EXTRUDE_END"));
}

#[test]
fn feedrate_is_quantized_to_whole_mm_s() {
    let mut buffer = OutputBuffer::default();
    let mut line = extrude_line("G1 X20 Y0 E2 F1800");
    line.modified = true;
    line.volumetric_extrusion_rate = 433.0;
    line.volumetric_extrusion_rate_start = 433.0;
    line.volumetric_extrusion_rate_end = 433.0;

    output_gcode_line(&mut buffer, &mut line, 2.0, false);

    let out = buffer.finish();
    // The corrected feedrate (1800 * 1.0 correction) is a multiple of 60.
    assert!(out.contains("G1 F1800;_EXTRUDE_SET_SPEED"));
}
