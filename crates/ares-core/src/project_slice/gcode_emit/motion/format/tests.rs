use super::{axis, extrusion, offset, ostream, z};
#[test]
fn zero_axis_words_remain_valid_gcode_numbers() {
    assert_eq!(axis(0.0), "0");
    assert_eq!(axis(-0.0), "0");
    assert_eq!(axis(1.230_4), "1.23");
    assert_eq!(axis(154.692_5), "154.693");
    assert_eq!(axis(-154.692_5), "-154.693");
}

#[test]
fn axis_omits_leading_zeroes_for_sub_unit_magnitudes() {
    // `GCodeFormatter::emit_axis` (GCodeWriter.cpp:1248-1265) strips the
    // leading zero on both signs: X0.75 -> X.75, Y-0.936 -> Y-.936.
    assert_eq!(axis(0.75), ".75");
    assert_eq!(axis(-0.936), "-.936");
}

#[test]
fn arc_offsets_keep_zero_and_omit_other_leading_zeroes() {
    assert_eq!(offset(0.0), "0");
    assert_eq!(offset(-0.0), "0");
    assert_eq!(offset(0.75), ".75");
    assert_eq!(offset(-0.75), "-.75");
}

#[test]
fn z_uses_axis_precision_and_omits_the_leading_zero() {
    assert_eq!(z(0.2), ".2");
    assert_eq!(z(1.995_6), "1.996");
}

#[test]
fn extrusion_uses_relative_e_precision_and_leading_zero_style() {
    assert_eq!(extrusion(0.0), "0");
    assert_eq!(extrusion(0.4), ".4");
    assert_eq!(extrusion(-0.4), "-.4");
    assert_eq!(extrusion(1.234_567), "1.23457");
}

#[test]
fn ostream_uses_six_significant_digits_without_axis_quantization() {
    // `GCodeWriter::set_jerk_xy` (`GCodeWriter.cpp:295`) writes jerk with
    // the stream default `<<`, not the axis quantizer.
    assert_eq!(ostream(1.962_4), "1.9624");
    assert_eq!(ostream(8.0), "8");
    assert_eq!(ostream(0.5), "0.5");
    // More than six significant digits round to the sixth.
    assert_eq!(ostream(1.962_399), "1.9624");
    assert_eq!(ostream(12.345_678), "12.3457");
    assert_eq!(ostream(0.0), "0");
}
