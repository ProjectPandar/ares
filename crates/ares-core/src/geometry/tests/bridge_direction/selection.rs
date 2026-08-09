use super::*;

const ZERO: u64 = 0x0000_0000_0000_0000;
const NEG_ZERO: u64 = 0x8000_0000_0000_0000;
const ONE: u64 = 0x3ff0_0000_0000_0000;
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

#[test]
fn task22o38_reversed_normals_remain_distinct_and_first_equal_cost_wins() {
    assert_output_bits(
        detect_lines(&[line(0, 0, 10, 0), line(0, 0, -10, 0)]),
        (NEG_ONE, NEG_ZERO, ZERO),
    );
    assert_output_bits(
        detect_lines(&[line(0, 0, -10, 0), line(0, 0, 10, 0)]),
        (ONE, NEG_ZERO, ZERO),
    );
}

#[test]
fn task22o38_axes_tie_exposes_occupied_bucket_front_insertion_and_strict_less() {
    assert_output_bits(
        detect_lines(&[line(0, 0, 10, 0), line(0, 0, 0, 10)]),
        (ZERO, NEG_ONE, 0x4024_0000_0000_0000),
    );
}

#[test]
fn task22o38_symmetric_eight_pins_pre_growth_bucket_iteration() {
    let edges = [
        line(0, 0, 5, 0),
        line(0, 0, 5, 5),
        line(0, 0, 0, 5),
        line(0, 0, -5, 5),
        line(0, 0, -5, 0),
        line(0, 0, -5, -5),
        line(0, 0, 0, -5),
        line(0, 0, 5, -5),
    ];
    assert_output_bits(
        detect_lines(&edges),
        (
            0x3fe6_a09e_667f_3bcc,
            0x3fe6_a09e_667f_3bcc,
            0x403c_48c6_001f_0abf,
        ),
    );
}

#[test]
fn task22o38_ninth_distinct_key_pins_eight_to_sixty_four_bucket_rehash() {
    let edges = [
        line(0, 0, 5, 0),
        line(0, 0, 5, 5),
        line(0, 0, 0, 5),
        line(0, 0, -5, 5),
        line(0, 0, -5, 0),
        line(0, 0, -5, -5),
        line(0, 0, 0, -5),
        line(0, 0, 5, -5),
        line(0, 0, 2, 1),
    ];
    assert_output_bits(
        detect_lines(&edges),
        (
            0x3fe6_a09e_667f_3bcc,
            0x3fe6_a09e_667f_3bcc,
            0x403c_fdca_f353_049d,
        ),
    );
}

#[test]
fn task22o38_post_rehash_collision_reverses_group_instead_of_preserving_it() {
    let edges = [
        line(0, 0, -3, 2),
        line(0, 0, -4, 0),
        line(0, 0, 9, 9),
        line(0, 0, -8, -2),
        line(0, 0, 2, 0),
        line(0, 0, 9, 6),
        line(0, 0, -9, -6),
        line(0, 0, 1, 0),
        line(0, 0, -7, 6),
        line(0, 0, 0, 5),
        line(0, 0, 1, 9),
        line(0, 0, -8, -2),
        line(0, 0, -8, -3),
        line(0, 0, 0, 1),
        line(0, 0, 2, -6),
    ];
    assert_output_bits(
        detect_lines(&edges),
        (
            0xbfea_a027_f059_dce0,
            0xbfe1_c01a_a03b_e895,
            0x4046_0ca1_130a_6aea,
        ),
    );
}
