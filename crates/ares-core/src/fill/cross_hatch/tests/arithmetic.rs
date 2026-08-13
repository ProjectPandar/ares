use super::super::{CrossHatchFillParams, fill_surface};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, Polyline};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn params(spacing: f64, overlap: f64, z: f64, angle: f32, density: f32) -> CrossHatchFillParams {
    CrossHatchFillParams {
        z,
        spacing,
        overlap,
        angle,
        density,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: 0.05,
        dont_sort: false,
    }
}

fn expected(paths: &[&[(i64, i64)]]) -> Vec<Polyline> {
    paths
        .iter()
        .map(|path| Polyline::new(path.iter().map(|&(x, y)| Point::new(x, y)).collect()))
        .collect()
}

#[test]
fn task22o45_offset_cast_completes_in_f64_on_both_scales() {
    let normal = fill_surface(
        &rectangle(0, 0, 200_000, 180_000),
        params(0.1, 0.050_100_499_9, 0.0, 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();
    let large = fill_surface(
        &rectangle(0, 0, 20_000, 18_000),
        params(0.1, 0.051_004_999_961_9, 0.0, 0.0, 1.0),
        CoordinateScale::LargeBed,
    )
    .unwrap();

    assert_eq!(
        normal,
        expected(&[
            &[(0, -100), (0, 180_100)],
            &[(100_000, -100), (100_000, 180_100)],
            &[(200_000, -100), (200_000, 180_100)],
        ])
    );
    assert_eq!(
        large,
        expected(&[
            &[(0, -100), (0, 18_101)],
            &[(10_000, -100), (10_000, 18_101)],
            &[(20_000, -100), (20_000, 18_101)],
        ])
    );
}

#[test]
fn task22o45_large_bed_line_spacing_truncates_before_density_multiplier() {
    let actual = fill_surface(
        &rectangle(0, 0, 600_000, 400_000),
        params(
            f64::from(f32::from_bits(0x3ed0_6cbe)),
            0.0,
            9.0,
            0.0,
            f32::from_bits(0x3e19_999a),
        ),
        CoordinateScale::LargeBed,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[(293_096, 20_354), (293_096, 379_646)]])
    );
}

#[test]
fn task22o45_density_0999_threshold_uses_widened_f32_neighbors() {
    let surface = rectangle(0, 0, 3_000_000, 2_000_000);
    let below = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, 0.0, f32::from_bits(0x3f7f_be76)),
        CoordinateScale::Normal,
    )
    .unwrap();
    let above = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, 0.0, f32::from_bits(0x3f7f_be77)),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        below,
        expected(&[
            &[(1_081_081, 500_000), (1_081_081, 1_500_000)],
            &[(2_162_162, 500_000), (2_162_162, 1_500_000)],
        ])
    );
    assert_eq!(
        above,
        expected(&[
            &[(1_001_000, 500_000), (1_001_000, 1_500_000)],
            &[(2_002_000, 500_000), (2_002_000, 1_500_000)],
        ])
    );
}

#[test]
fn task22o45_density_03_threshold_uses_widened_f32_neighbors() {
    let surface = rectangle(0, 0, 7_000_000, 4_000_000);
    let below = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, 0.0, f32::from_bits(0x3e99_9999)),
        CoordinateScale::Normal,
    )
    .unwrap();
    let above = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, 0.0, f32::from_bits(0x3e99_999a)),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        below,
        expected(&[&[
            (3_585_200, 3_500_000),
            (3_367_956, 2_032_042),
            (3_367_956, 1_567_957),
            (3_526_004, 500_000),
        ]])
    );
    assert_eq!(
        above,
        expected(&[&[(3_599_999, 500_000), (3_599_999, 3_500_000)]])
    );
}

#[test]
fn task22o45_low_density_repeat_ratio_clamps_to_one_fifth() {
    let actual = fill_surface(
        &rectangle(0, 0, 25_000_000, 15_000_000),
        params(0.1, 0.0, 0.0, 0.0, 0.01),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[
            &[
                (24_950_000, 8_996_154),
                (18_090_000, 12_690_000),
                (14_310_000, 12_690_000),
                (7_290_000, 8_910_000),
                (3_510_000, 8_910_000),
                (50_000, 10_773_077),
            ],
            &[
                (10_707_143, 50_000),
                (7_290_000, 1_890_000),
                (3_510_000, 1_890_000),
            ],
            &[(24_950_000, 1_803_846), (21_692_857, 50_000)],
        ])
    );
}

#[test]
fn task22o45_negative_contour_minimum_uses_floor_grid_alignment() {
    let actual = fill_surface(
        &rectangle(-2_500_000, -2_500_000, 3_500_000, 3_500_000),
        params(1.0, 0.0, 0.0, 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[
            &[(-2_000_000, -2_000_000), (-2_000_000, 3_000_000)],
            &[(-1_000_000, -2_000_000), (-1_000_000, 3_000_000)],
            &[(0, -2_000_000), (0, 3_000_000)],
            &[(1_000_000, -2_000_000), (1_000_000, 3_000_000)],
            &[(2_000_000, -2_000_000), (2_000_000, 3_000_000)],
            &[(3_000_000, -2_000_000), (3_000_000, 3_000_000)],
        ])
    );
}

#[test]
fn task22o45_angle_threshold_uses_f64_epsilon_after_f32_abs() {
    let surface = rectangle(-2_500_000, -1_500_000, 3_500_000, 2_500_000);
    let below = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, f32::from_bits(0x38d1_b717), 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();
    let above = fill_surface(
        &surface,
        params(1.0, 0.0, 0.0, f32::from_bits(0x38d1_b718), 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        below,
        expected(&[
            &[(-2_000_000, -1_000_000), (-2_000_000, 2_000_000)],
            &[(-1_000_000, -1_000_000), (-1_000_000, 2_000_000)],
            &[(0, -1_000_000), (0, 2_000_000)],
            &[(1_000_000, -1_000_000), (1_000_000, 2_000_000)],
            &[(2_000_000, -1_000_000), (2_000_000, 2_000_000)],
            &[(3_000_000, -1_000_000), (3_000_000, 2_000_000)],
        ])
    );
    assert_eq!(
        above,
        expected(&[
            &[(-999_900, -1_000_000), (-1_000_200, 2_000_000)],
            &[(100, -1_000_000), (-200, 2_000_000)],
            &[(1_000_100, -1_000_000), (999_800, 2_000_000)],
            &[(2_000_100, -1_000_000), (1_999_800, 2_000_000)],
            &[(3_000_000, 0), (2_999_800, 2_000_000)],
            &[(-1_999_900, -1_000_000), (-2_000_000, 0)],
        ])
    );
}

#[test]
fn task22o45_fixed_point_construction_rounds_halves_away_from_zero() {
    let actual = fill_surface(
        &rectangle(0, 0, 30, 20),
        params(0.000_1, 0.0, 0.000_01, 0.0, 1.0),
        CoordinateScale::LargeBed,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[(21, 5), (21, 6), (19, 15), (11, 15), (9, 6), (9, 5)]])
    );
}

#[test]
fn task22o45_negative_shifted_z_uses_floor_remainder() {
    let actual = fill_surface(
        &rectangle(0, 0, 3_000_000, 2_000_000),
        params(1.0, 0.0, -1.0, 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[
            (2_500_000, 825_000),
            (2_325_000, 825_000),
            (1_675_000, 1_175_000),
            (1_325_000, 1_175_000),
            (675_000, 825_000),
            (500_000, 825_000),
        ]])
    );
}

#[test]
fn task22o45_positive_fractional_phase_narrows_to_zero_before_direction() {
    let actual = fill_surface(
        &rectangle(0, 0, 3_000_000, 2_000_000),
        params(1.0, 0.0, f64::from_bits(0x3fdf_fffd_e721_0be9), 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[
            (2_500_000, 949_999),
            (2_449_999, 949_999),
            (1_550_001, 1_050_001),
            (1_449_999, 1_050_001),
            (550_001, 949_999),
            (500_000, 949_999),
        ]])
    );
}

#[test]
fn task22o45_exact_trans_zero_selects_transform() {
    let actual = fill_surface(
        &rectangle(0, 0, 3_000_000, 2_000_000),
        params(1.0, 0.0, f64::from_bits(0x3fb9_9999_9999_9999), 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[
            &[
                (1_050_000, 1_500_000),
                (1_050_000, 1_450_000),
                (950_000, 550_000),
                (950_000, 500_000),
            ],
            &[
                (1_950_000, 1_500_000),
                (1_950_000, 1_450_000),
                (2_050_000, 550_000),
                (2_050_000, 500_000),
            ],
        ])
    );
}

#[test]
fn task22o45_exact_half_progress_selects_backward_transform() {
    let actual = fill_surface(
        &rectangle(0, 0, 3_000_000, 2_000_000),
        params(1.0, 0.0, f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[
            (2_500_000, 700_000),
            (2_200_000, 700_000),
            (1_800_000, 1_300_000),
            (1_200_000, 1_300_000),
            (800_000, 700_000),
            (500_000, 700_000),
        ]])
    );
}

#[test]
fn task22o45_short_remnant_at_exact_threshold_survives() {
    let actual = fill_surface(
        &rectangle(0, 0, 2_500_000, 2_250_000),
        params(1.25, 0.0, 0.0, 0.0, 1.0),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        expected(&[&[(1_250_000, 625_000), (1_250_000, 1_625_000)]])
    );
}
