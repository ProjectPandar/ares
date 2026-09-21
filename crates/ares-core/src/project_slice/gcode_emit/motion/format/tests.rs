use super::{axis, extrusion, offset, z};
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
