use super::super::{CrossHatchFillParams, fill_surface};
use super::support::point;
use crate::geometry::{CoordinateScale, ExPolygon, Polygon, Polyline};

#[test]
fn task22o45_public_repeat_ratio_uses_f32_exponential_before_widening() {
    let surface = ExPolygon::new(
        Polygon::new(vec![
            point(0, 0),
            point(7_000_000, 0),
            point(7_000_000, 4_000_000),
            point(0, 4_000_000),
        ]),
        Vec::new(),
    );
    let params = CrossHatchFillParams {
        z: 41.0,
        spacing: 1.0,
        overlap: 0.0,
        angle: 0.0,
        density: f32::from_bits(0x3e19_999a),
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: f32::from_bits(0x3d4c_cccd),
        dont_sort: false,
    };

    let actual = fill_surface(&surface, params, CoordinateScale::Normal).unwrap();

    assert_eq!(
        actual,
        vec![Polyline::new(vec![
            point(6_500_000, 545_397),
            point(5_176_549, 1_576_550),
            point(2_023_450, 1_576_550),
        ])]
    );
}
