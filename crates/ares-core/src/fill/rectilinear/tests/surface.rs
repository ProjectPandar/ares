use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

use super::super::{
    MonotonicFillParams, fast_round_up, fill_monotonic_surface, surface::scaled_offsets,
};

fn scaled_rectangle() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 800_000),
            Point::new(0, 800_000),
        ]),
        Vec::new(),
    )
}

fn params() -> MonotonicFillParams {
    MonotonicFillParams {
        spacing: 0.2,
        overlap: 0.0,
        density: 1.0,
        angle: -std::f32::consts::FRAC_PI_2,
        layer_index: 0,
        thickness_layers: 1,
        fixed_angle: true,
        bridge_angle: None,
        reference_point: Point::new(0, 0),
        dont_adjust: false,
        anchor_length_max: 1_000.0,
        link_max_length: 0.6,
    }
}

#[test]
fn task22o89_full_solid_rectangle_emits_repeatable_scaled_polylines() {
    let source = scaled_rectangle();
    let before = source.clone();
    let first = fill_monotonic_surface(&source, params(), CoordinateScale::Normal).unwrap();
    let second = fill_monotonic_surface(&source, params(), CoordinateScale::Normal).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first
            .polylines
            .iter()
            .map(|polyline| polyline.points().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            Point::new(120_050, 10_000),
            Point::new(120_050, 700_000),
            Point::new(360_050, 700_000),
            Point::new(360_050, 100_000),
            Point::new(600_050, 100_000),
            Point::new(600_050, 700_000),
            Point::new(840_050, 700_000),
            Point::new(840_050, 10_000),
        ]]
    );
    assert!(first.polylines.iter().all(|polyline| polyline.is_valid()));
    assert_eq!(first.spacing.to_bits(), 0.24_f32.to_bits());
    assert_eq!(source, before);
}

#[test]
fn large_scan_origins_follow_orca_float_horizontal_alignment() {
    let source = ExPolygon::new(
        Polygon::new(vec![
            Point::new(-50_000_000, 0),
            Point::new(-49_000_000, 0),
            Point::new(-49_000_000, 800_000),
            Point::new(-50_000_000, 800_000),
        ]),
        Vec::new(),
    );

    let output = fill_monotonic_surface(&source, params(), CoordinateScale::Normal).unwrap();

    assert_eq!(output.polylines[0].points()[0].x(), -49_879_952);
}

#[test]
fn unadjusted_solid_fill_aligns_scanlines_to_the_object_grid() {
    let source = ExPolygon::new(
        Polygon::new(vec![
            Point::new(100_000, 0),
            Point::new(1_100_000, 0),
            Point::new(1_100_000, 800_000),
            Point::new(100_000, 800_000),
        ]),
        Vec::new(),
    );
    let mut fill_params = params();
    fill_params.dont_adjust = true;

    let output = fill_monotonic_surface(&source, fill_params, CoordinateScale::Normal).unwrap();
    let mut xs = output
        .polylines
        .iter()
        .flat_map(|polyline| polyline.points().iter().map(|point| point.x()))
        .collect::<Vec<_>>();
    xs.sort_unstable();
    xs.dedup();

    assert_eq!(xs, [300_050, 500_050, 700_050, 900_050]);
}

#[test]
fn unadjusted_fill_preserves_source_flow_spacing() {
    let mut fill_params = params();
    fill_params.spacing = 0.357_079_632_679_489_65;
    fill_params.dont_adjust = true;

    let output =
        fill_monotonic_surface(&scaled_rectangle(), fill_params, CoordinateScale::Normal).unwrap();

    assert_eq!(
        output.spacing.to_bits(),
        (fill_params.spacing as f32).to_bits()
    );
}

#[test]
fn task22o89_fixed_angle_and_layer_alternation_select_distinct_directions() {
    let source = scaled_rectangle();
    let fixed = fill_monotonic_surface(&source, params(), CoordinateScale::Normal).unwrap();
    let mut alternating = params();
    alternating.fixed_angle = false;
    alternating.layer_index = 1;
    let rotated = fill_monotonic_surface(&source, alternating, CoordinateScale::Normal).unwrap();

    assert_ne!(fixed, rotated);
}

#[test]
fn rectilinear_offsets_truncate_to_source_coordinate_grid() {
    let offsets = scaled_offsets(CoordinateScale::Normal, 0.1, 0.500_542_342_662_811_3).unwrap();

    assert_eq!(offsets, (74_972.0, -150_271.0));
}

#[test]
fn rotated_coordinates_use_orca_half_up_rounding() {
    assert_eq!(fast_round_up(0.5), 1.0);
    assert_eq!(fast_round_up(-0.5), 0.0);
    assert_eq!(fast_round_up(-1.5), -1.0);
    assert_eq!(fast_round_up(0.499_999_999_999_999_94), 0.0);
}
