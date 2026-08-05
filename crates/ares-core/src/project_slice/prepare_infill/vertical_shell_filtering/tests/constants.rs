use crate::{
    geometry::CoordinateScale,
    project_slice::prepare_infill::vertical_shell_filtering::{epsilon_bits, threshold_bits},
};

#[test]
fn task22o23_normal_constant_bits_preserve_source_mixed_precision() {
    assert_eq!(
        threshold_bits(400_000, CoordinateScale::Normal),
        [
            1_221_399_551,
            1_500_000,
            u64::from(1_500_000_f32.to_bits()),
            8_000_000,
            u64::from(8_000_000_f32.to_bits()),
            u64::from(629_999_953_125.0_f32.to_bits()),
            u64::from(3_359_999_750_000.0_f32.to_bits()),
            0x4059_0000_0000_0001,
        ]
    );
    assert_eq!(epsilon_bits(CoordinateScale::Normal), 0x42c8_0000);
}

#[test]
fn task22o23_large_bed_constant_bits_preserve_selected_scale() {
    assert_eq!(
        threshold_bits(400_000, CoordinateScale::LargeBed),
        [
            1_221_399_551,
            150_000,
            1_209_170_944,
            799_999,
            1_229_148_144,
            1_365_946_746,
            1_385_985_605,
            0x4024_0000_0000_0000,
        ]
    );
    assert_eq!(epsilon_bits(CoordinateScale::LargeBed), 0x4120_0000);
}

#[test]
fn task22o23_odd_and_above_exact_f32_spacing_use_the_shared_one_cast_minimum() {
    for spacing in [400_001, 16_777_217] {
        assert_eq!(
            threshold_bits(spacing, CoordinateScale::Normal)[0],
            u64::from(((spacing as f32) * 1.05_f32).to_bits())
        );
    }
}
