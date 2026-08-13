use super::super::scale::{inflate_bbox_round_delta, scaled_coord_f64, scaled_f32, scaled_f64};
use super::{FillConnectionParams, connect_infill, point};
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, Polygon, Polyline};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            point(min_x, min_y),
            point(max_x, min_y),
            point(max_x, max_y),
            point(min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn bounds(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> BoundingBox {
    BoundingBox::from_expolygon(&rectangle(min_x, min_y, max_x, max_y)).unwrap()
}

fn ksr_spacing() -> f64 {
    f64::from(f32::from_bits(0x3ed0_6cbe))
}

fn params() -> FillConnectionParams {
    FillConnectionParams {
        anchor_length: f32::from_bits(0x3fd0_6cbe),
        anchor_length_max: 20.0,
        multiline: 1,
        dont_sort: false,
    }
}

#[test]
fn task22o44_ksr_f32_anchor_and_spacing_keep_fractional_dual_scale_values() {
    let anchor = f32::from_bits(0x3fd0_6cbe);
    let spacing = f64::from(f32::from_bits(0x3ed0_6cbe));

    assert_eq!(
        [
            scaled_f32(anchor, CoordinateScale::Normal).to_bits(),
            scaled_f64(spacing, CoordinateScale::Normal).to_bits(),
            scaled_f32(anchor, CoordinateScale::LargeBed).to_bits(),
            scaled_f64(spacing, CoordinateScale::LargeBed).to_bits(),
        ],
        [
            0x4138_d89e_8c57_0000,
            0x4118_d89e_8c57_0000,
            0x4103_e07e_d6ac_0000,
            0x40e3_e07e_d6ac_0000,
        ]
    );
}

#[test]
fn task22o44_large_bed_ten_millimeter_grid_resolution_truncates_completed_scale() {
    assert_eq!(
        scaled_coord_f64(10.0, CoordinateScale::LargeBed),
        Ok(999_999)
    );
}

#[test]
fn task22o44_bbox_half_delta_rounds_before_inflating_positive_and_negative_bounds() {
    let positive = inflate_bbox_round_delta(bounds(2, 1, 8, 4), 0.5).unwrap();
    let negative = inflate_bbox_round_delta(bounds(-8, -4, -2, -1), 0.5).unwrap();

    assert_eq!(
        [(positive.min, positive.max), (negative.min, negative.max)],
        [(point(1, 0), point(9, 5)), (point(-9, -5), point(-1, 0)),]
    );
}

#[test]
fn task22o44_bbox_inflation_checks_both_coordinate_limits_after_rounding() {
    assert_eq!(
        [
            inflate_bbox_round_delta(bounds(i64::MIN, -5, i64::MIN + 10, 5), 0.5),
            inflate_bbox_round_delta(bounds(-10, -5, i64::MAX, 5), 0.5),
        ],
        [
            Err(ClipperError::CoordinateOutOfRange),
            Err(ClipperError::CoordinateOutOfRange),
        ]
    );
}

#[test]
fn task22o44_negative_limited_hooks_truncate_fractional_coordinates_toward_zero() {
    let boundary = rectangle(0, -8_000_000, 12_000_000, 0);
    let before = boundary.clone();
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(0, -6_000_000),
            point(12_000_000, -5_000_000),
        ])],
        &boundary,
        ksr_spacing(),
        params(),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        vec![Polyline::new(vec![
            point(0, -4_371_681),
            point(0, -6_000_000),
            point(12_000_000, -5_000_000),
            point(12_000_000, -3_371_681),
        ])]
    );
    assert_eq!(boundary, before);
}

#[test]
fn task22o44_projection_grid_margin_overflow_returns_direct_error_without_boundary_mutation() {
    let min_x = i64::MIN + 115;
    let max_x = min_x + 1_000_000;
    let boundary = rectangle(min_x, 0, max_x, 1_000_000);
    let before = boundary.clone();
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(min_x, 250_000),
            point(max_x, 750_000),
        ])],
        &boundary,
        ksr_spacing(),
        params(),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(boundary, before);
}

#[test]
fn task22o44_touching_bbox_overflow_returns_direct_error_without_boundary_mutation() {
    let min_x = i64::MIN + 200;
    let max_x = min_x + 1_000_000;
    let boundary = rectangle(min_x, 0, max_x, 1_000_000);
    let before = boundary.clone();
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(min_x, 250_000),
            point(max_x, 750_000),
        ])],
        &boundary,
        ksr_spacing(),
        params(),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(boundary, before);
}

#[test]
fn task22o44_negative_trace_error_precedes_positive_trace_grid_visit() {
    let boundary = rectangle(-1_000_000, -1_000_000, 1_000_000, 1_000_000);
    let before = boundary.clone();
    // The negative trace overflows during checked conversion. The positive trace converts but
    // misses this grid, so visiting it first trips the production containment assertion.
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(10_000_000, i64::MAX - 100),
            point(0, i64::MAX - 100),
        ])],
        &boundary,
        ksr_spacing(),
        params(),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(boundary, before);
}
