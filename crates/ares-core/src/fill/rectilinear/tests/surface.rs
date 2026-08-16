use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

use super::super::{MonotonicFillParams, fill_monotonic_surface};

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
            .iter()
            .map(|polyline| polyline.points().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            Point::new(120_050, 9_999),
            Point::new(120_050, 700_000),
            Point::new(360_050, 700_000),
            Point::new(360_050, 100_000),
            Point::new(600_050, 100_000),
            Point::new(600_050, 700_000),
            Point::new(840_050, 700_000),
            Point::new(840_050, 9_999),
        ]]
    );
    assert!(first.iter().all(|polyline| polyline.is_valid()));
    assert_eq!(source, before);
}

#[test]
fn task22o124_monotonic_keeps_its_configured_direction_between_layers() {
    let source = scaled_rectangle();
    let first = fill_monotonic_surface(&source, params(), CoordinateScale::Normal).unwrap();
    let mut next_layer = params();
    next_layer.fixed_angle = false;
    next_layer.layer_index = 1;
    let second = fill_monotonic_surface(&source, next_layer, CoordinateScale::Normal).unwrap();

    assert_eq!(first, second);
}
