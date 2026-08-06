use crate::geometry::{CoordinateScale, RegionExpansionParameters};

fn assert_bits(
    actual: RegionExpansionParameters,
    floats: [u32; 4],
    count: usize,
    doubles: [u64; 2],
) {
    assert_eq!(actual.tiny_expansion.to_bits(), floats[0]);
    assert_eq!(actual.initial_step.to_bits(), floats[1]);
    assert_eq!(actual.other_step.to_bits(), floats[2]);
    assert_eq!(actual.num_other_steps, count);
    assert_eq!(actual.max_inflation.to_bits(), floats[3]);
    assert_eq!(actual.arc_tolerance.to_bits(), doubles[0]);
    assert_eq!(actual.shortest_edge_length.to_bits(), doubles[1]);
}

#[test]
fn task22o27_normal_parameter_arithmetic_is_bit_exact() {
    assert_bits(
        RegionExpansionParameters::build(1_000_000.0, 100_000.0, 5, CoordinateScale::Normal),
        [0x4743_5000, 0x4867_ef00, 0x4867_ef00, 0x4986_4700],
        3,
        [0x40f8_6a00_0000_0001, 0x4092_8e00_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(10_000.0, 100_000.0, 5, CoordinateScale::Normal),
        [0x44fa_0000, 0x45fa_0000, 0x45fa_0000, 0x462b_e000],
        0,
        [0x40f8_6a00_0000_0001, 0x4044_0000_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(600_000.0, 10_000.0, 5, CoordinateScale::Normal),
        [0x4743_5000, 0x4886_4700, 0x4886_4700, 0x4921_2200],
        1,
        [0x40f8_6a00_0000_0001, 0x4095_7c00_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(120_000.0, 100_000.0, 5, CoordinateScale::Normal),
        [0x46bb_8000, 0x47bb_8000, 0x47bb_8000, 0x4800_e800],
        0,
        [0x40f8_6a00_0000_0001, 0x407e_0000_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(
            1_718_556_032.0,
            66.356_98,
            8_812,
            CoordinateScale::Normal,
        ),
        [0x4743_5000, 0x4843_5315, 0x4843_5315, 0x4ee1_5ac9],
        8_591,
        [0x40f8_6a00_0000_0001, 0x408f_407e_3d70_a3d7],
    );
    assert_bits(
        RegionExpansionParameters::build(1_638_649_984.0, 1.0, 10_000, CoordinateScale::Normal),
        [0x4743_5000, 0x4843_561a, 0x4843_561a, 0x4ed6_e064],
        8_191,
        [0x40f8_6a00_0000_0001, 0x408f_40f9_eb85_1eb9],
    );
}

#[test]
fn task22o27_large_bed_parameter_arithmetic_is_bit_exact() {
    assert_bits(
        RegionExpansionParameters::build(100_000.0, 10_000.0, 5, CoordinateScale::LargeBed),
        [0x459c_4000, 0x46b9_8c00, 0x46b9_8c00, 0x47d6_d800],
        3,
        [0x40c3_8800_0000_0000, 0x405d_b000_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(1_000.0, 10_000.0, 5, CoordinateScale::LargeBed),
        [0x4348_0000, 0x4448_0000, 0x4448_0000, 0x4489_8000],
        0,
        [0x40c3_8800_0000_0000, 0x4010_0000_0000_0000],
    );
    assert_bits(
        RegionExpansionParameters::build(60_000.0, 1_000.0, 5, CoordinateScale::LargeBed),
        [0x459c_4000, 0x46d6_d800, 0x46d6_d800, 0x4780_e800],
        1,
        [0x40c3_8800_0000_0000, 0x4061_3000_0000_0000],
    );
}

#[test]
#[should_panic]
fn task22o27_parameters_assert_positive_full_expansion() {
    RegionExpansionParameters::build(0.0, 1.0, 1, CoordinateScale::Normal);
}

#[test]
#[should_panic]
fn task22o27_parameters_assert_positive_step() {
    RegionExpansionParameters::build(1.0, 0.0, 1, CoordinateScale::Normal);
}

#[test]
#[should_panic]
fn task22o27_parameters_assert_positive_maximum_steps() {
    RegionExpansionParameters::build(1.0, 1.0, 0, CoordinateScale::Normal);
}
