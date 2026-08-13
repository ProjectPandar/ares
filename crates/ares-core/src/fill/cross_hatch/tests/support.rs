use super::super::CrossHatchFillParams;
use crate::geometry::{ExPolygon, Point, Polygon};

pub(super) fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

pub(super) fn raw_surface() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            point(-8_300_000, -5_700_000),
            point(-1_200_000, -5_400_000),
            point(-1_000_000, -130_000),
            point(1_100_000, -110_000),
            point(1_300_000, -4_800_000),
            point(9_400_000, -4_500_000),
            point(9_000_000, 6_200_000),
            point(1_200_000, 5_800_000),
            point(1_000_000, 140_000),
            point(-1_100_000, 120_000),
            point(-1_400_000, 5_200_000),
            point(-8_000_000, 5_500_000),
            point(-8_800_000, 700_000),
        ]),
        vec![Polygon::new(vec![
            point(2_800_000, -1_300_000),
            point(2_900_000, 2_100_000),
            point(5_400_000, 1_700_000),
            point(5_100_000, -1_500_000),
        ])],
    )
}

pub(super) fn large_bed_surface() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            point(-830_000, -570_000),
            point(-119_999, -540_000),
            point(-99_999, -13_000),
            point(110_000, -11_000),
            point(130_000, -479_999),
            point(940_000, -449_999),
            point(899_999, 620_000),
            point(119_999, 579_999),
            point(99_999, 14_000),
            point(-110_000, 11_999),
            point(-139_999, 520_000),
            point(-799_999, 550_000),
            point(-880_000, 69_999),
        ]),
        vec![Polygon::new(vec![
            point(279_999, -130_000),
            point(289_999, 210_000),
            point(540_000, 169_999),
            point(509_999, -150_000),
        ])],
    )
}

pub(super) fn ksr_params() -> CrossHatchFillParams {
    CrossHatchFillParams {
        z: f64::from_bits(0x4022_0000_0000_0000),
        spacing: f64::from(f32::from_bits(0x3ed0_6cbe)),
        overlap: f64::from_bits(0x0000_0000_0000_0000),
        angle: f32::from_bits(0x3f49_0fdb),
        density: f32::from_bits(0x3e19_999a),
        multiline: 1,
        anchor_length: f32::from_bits(0x3fd0_6cbe),
        anchor_length_max: f32::from_bits(0x41a0_0000),
        dont_sort: false,
    }
}
