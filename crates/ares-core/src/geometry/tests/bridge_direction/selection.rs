use super::*;

const ZERO: u64 = 0x0000_0000_0000_0000;
const NEG_ZERO: u64 = 0x8000_0000_0000_0000;
const NEG_ONE: u64 = 0xbff0_0000_0000_0000;

fn detect_lines(lines: &[Line]) -> ((f64, f64), f64) {
    detect_bridging_direction(lines, &[], CoordinateScale::Normal)
}

#[test]
fn task22o38_horizontal_and_vertical_lines_preserve_normal_rotation_and_zero_cost() {
    assert_output_bits(
        detect_lines(&[line(0, 0, 10, 0)]),
        (NEG_ONE, NEG_ZERO, ZERO),
    );
    assert_output_bits(detect_lines(&[line(0, 0, 0, 10)]), (ZERO, NEG_ONE, ZERO));
}

#[test]
fn task22o38_single_oblique_line_pins_f64_scalar_division_and_residual_cost() {
    assert_output_bits(
        detect_lines(&[line(0, 0, 3, 4)]),
        (
            0xbfe3_3333_3333_3333,
            0xbfe9_9999_9999_999a,
            0x3cc0_0000_0000_0000,
        ),
    );
}

#[test]
fn task22o38_zero_length_line_retains_zero_normal_and_negative_zero_rotation() {
    assert_output_bits(detect_lines(&[line(7, 9, 7, 9)]), (ZERO, NEG_ZERO, ZERO));
}

#[test]
fn task22o38_multiple_candidates_preserve_input_accumulation_and_strict_minimum() {
    let edges = [line(0, 0, 9, 2), line(0, 0, 1, 7), line(4, 3, -5, 6)];
    assert_output_bits(
        detect_lines(&edges),
        (
            0xbfef_3cec_a548_973c,
            0xbfcb_c460_92eb_3118,
            0x4026_fe9f_f9aa_c4a8,
        ),
    );
}

#[test]
fn task22o38_duplicate_quantized_key_keeps_first_emplaced_normal() {
    assert_output_bits(
        detect_lines(&[line(0, 0, -50, -50), line(0, 0, -46, -46)]),
        (0x3fe6_a09e_667f_3bcc, 0x3fe6_a09e_667f_3bcc, ZERO),
    );
    assert_output_bits(
        detect_lines(&[line(0, 0, -46, -46), line(0, 0, -50, -50)]),
        (0x3fe6_a09e_667f_3bcd, 0x3fe6_a09e_667f_3bcd, ZERO),
    );
}
