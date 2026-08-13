use crate::geometry::{CoordinateScale, Point};

use super::{line, polygon};
use crate::project_slice::prepare_infill::bridge_over_infill::automatic_bridge_angle::{
    counted_directions_for_test, sampled_points_for_test,
};

#[test]
fn task22o51_integer_scaled_threshold_is_strict_at_both_scales() {
    for (scale, threshold, midpoint) in [
        (CoordinateScale::Normal, 2_000_000, 1_000_000),
        (CoordinateScale::LargeBed, 199_999, 100_000),
    ] {
        let exact = [polygon(&[(0, 0), (threshold, 0)])];
        assert!(sampled_points_for_test(&exact, scale).is_empty());

        let over = [polygon(&[(0, 0), (threshold + 1, 0)])];
        assert_eq!(
            sampled_points_for_test(&over, scale),
            vec![Point::new(0, 0), Point::new(midpoint, 0)]
        );
    }
}

#[test]
fn task22o51_polygon_scope_reset_remainder_and_missing_closing_edge_match_source() {
    let no_closing = [polygon(&[(0, 0), (2_100_000, 0), (2_100_000, 100_000)])];
    assert_eq!(
        sampled_points_for_test(&no_closing, CoordinateScale::Normal),
        vec![Point::new(0, 0), Point::new(1_050_000, 0)]
    );

    let no_remainder = [polygon(&[
        (0, 0),
        (1_500_000, 0),
        (2_500_000, 0),
        (4_000_000, 0),
    ])];
    assert_eq!(
        sampled_points_for_test(&no_remainder, CoordinateScale::Normal),
        vec![Point::new(1_500_000, 0)]
    );

    let separate = [
        polygon(&[(0, 0), (1_500_000, 0)]),
        polygon(&[(1_500_000, 0), (3_000_000, 0)]),
    ];
    assert!(sampled_points_for_test(&separate, CoordinateScale::Normal).is_empty());
}

#[test]
fn task22o51_eigen_normalization_f32_step_and_nearest_ownership_match_oracle() {
    let anchors = [
        line(0, 0, -3_000_000, -4_000_000),
        line(2_000_000, -3_000_000, 2_000_000, 3_000_000),
    ];
    let normal = [polygon(&[(11, -17), (2_345_690, 765_414)])];
    let before_normal = normal.clone();
    let before_anchors = anchors;
    assert_eq!(
        sampled_points_for_test(&normal, CoordinateScale::Normal),
        vec![Point::new(11, -17), Point::new(1_172_850, 382_698)]
    );
    assert_eq!(
        counted_directions_for_test(&normal, &anchors, CoordinateScale::Normal),
        vec![(0x4003_fc17_6b7a_8560, 1), (0x4009_21fb_5444_2d18, 1)]
    );

    let large = [polygon(&[(11, -17), (234_580, 76_537)])];
    assert_eq!(
        sampled_points_for_test(&large, CoordinateScale::LargeBed),
        vec![Point::new(11, -17), Point::new(117_295, 38_260)]
    );
    assert_eq!(
        counted_directions_for_test(&large, &anchors, CoordinateScale::LargeBed),
        vec![(0x4003_fc17_6b7a_8560, 2)]
    );
    assert_eq!(normal, before_normal);
    assert_eq!(anchors, before_anchors);
}
