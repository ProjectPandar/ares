use super::*;

const ZERO: u64 = 0x0000_0000_0000_0000;
const ONE: u64 = 0x3ff0_0000_0000_0000;

#[test]
fn task22o38_empty_and_degenerate_overhangs_use_exact_pc2_zero_fallback() {
    assert_output_bits(
        detect_bridging_direction(&[], &[], CoordinateScale::Normal),
        (ONE, ZERO, ZERO),
    );
    assert_output_bits(
        detect_polygon(&[(0, 0), (1_000_000, 0)], CoordinateScale::Normal),
        (ONE, ZERO, ZERO),
    );
    assert_output_bits(
        detect_polygon(
            &[(0, 0), (1_000_000, 0), (2_000_000, 0)],
            CoordinateScale::Normal,
        ),
        (ONE, ZERO, ZERO),
    );
}

#[test]
fn task22o38_reversed_nonpositive_area_uses_exact_pc2_zero_fallback() {
    assert_output_bits(
        detect_polygon(
            &[
                (0, 1_000_000),
                (3_000_000, 1_000_000),
                (3_000_000, 0),
                (0, 0),
            ],
            CoordinateScale::Normal,
        ),
        (ONE, ZERO, ZERO),
    );
}

#[test]
fn task22o38_axis_and_equal_variance_covariance_branch_orders_pc2_literally() {
    for (points, scale) in [
        (
            vec![
                (0, 0),
                (3_000_000, 0),
                (3_000_000, 1_000_000),
                (0, 1_000_000),
            ],
            CoordinateScale::Normal,
        ),
        (
            vec![(0, 0), (100_000, 0), (100_000, 100_000), (0, 100_000)],
            CoordinateScale::LargeBed,
        ),
    ] {
        assert_output_bits(detect_polygon(&points, scale), (ZERO, ONE, ZERO));
    }
}

#[test]
fn task22o38_normal_and_large_scale_oblique_and_triangle_oracles_are_complete() {
    assert_output_bits(
        detect_polygon(
            &[
                (0, 0),
                (3_000_000, 1_000_000),
                (2_000_000, 4_000_000),
                (-1_000_000, 3_000_000),
            ],
            CoordinateScale::Normal,
        ),
        (ONE, ZERO, ZERO),
    );
    assert_output_bits(
        detect_polygon(
            &[(0, 0), (5_000_000, 1_000_000), (1_000_000, 3_000_000)],
            CoordinateScale::Normal,
        ),
        (ZERO, ONE, ZERO),
    );
    assert_output_bits(
        detect_polygon(
            &[
                (0, 0),
                (300_000, 100_000),
                (200_000, 400_000),
                (-100_000, 300_000),
            ],
            CoordinateScale::LargeBed,
        ),
        (ONE, ZERO, ZERO),
    );
    assert_output_bits(
        detect_polygon(
            &[(0, 0), (500_000, 100_000), (100_000, 300_000)],
            CoordinateScale::LargeBed,
        ),
        (ZERO, ONE, ZERO),
    );
}

#[test]
fn task22o38_non_axis_pca_pins_f32_eigen_and_normalization_bits_at_both_scales() {
    const RIGHT: (u64, u64, u64) = (0x3fc0_a1a1_4000_0000, 0x3fef_ba8e_4000_0000, ZERO);
    const IRREGULAR: (u64, u64, u64) = (0x3fc9_44d8_4000_0000, 0x3fef_5ec8_c000_0000, ZERO);

    assert_output_bits(
        detect_polygon(
            &[(0, 0), (4_000_000, 0), (0, 1_000_000)],
            CoordinateScale::Normal,
        ),
        RIGHT,
    );
    assert_output_bits(
        detect_polygon(
            &[
                (0, 0),
                (5_000_000, 0),
                (4_000_000, 2_000_000),
                (1_000_000, 3_500_000),
                (-500_000, 1_000_000),
            ],
            CoordinateScale::Normal,
        ),
        IRREGULAR,
    );
    assert_output_bits(
        detect_polygon(
            &[(0, 0), (400_000, 0), (0, 100_000)],
            CoordinateScale::LargeBed,
        ),
        RIGHT,
    );
    assert_output_bits(
        detect_polygon(
            &[
                (0, 0),
                (500_000, 0),
                (400_000, 200_000),
                (100_000, 350_000),
                (-50_000, 100_000),
            ],
            CoordinateScale::LargeBed,
        ),
        IRREGULAR,
    );
}

#[test]
fn task22o38_covariance_threshold_is_strict_and_nonzero_pc2_normalizes_in_f32() {
    assert_output_bits(
        detect_polygon(&[(0, 0), (30_000, 0), (0, 30_000)], CoordinateScale::Normal),
        (ZERO, ONE, ZERO),
    );
    assert_output_bits(
        detect_polygon(&[(0, 0), (90_000, 0), (0, 90_000)], CoordinateScale::Normal),
        (0x3fe6_a09e_6000_0000, 0x3fe6_a09e_6000_0000, ZERO),
    );
}

#[test]
#[should_panic]
fn task22o38_trusted_empty_polygon_panics() {
    let empty = Polygon::new(Vec::new());
    let _ = detect_bridging_direction(&[], &[empty], CoordinateScale::Normal);
}
