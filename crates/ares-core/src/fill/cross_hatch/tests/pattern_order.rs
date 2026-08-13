use super::super::{CrossHatchFillParams, fill_surface};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, Polyline};

const TRANSFORM_NEGATIVE_Z_BITS: u64 = 0x3ef2_dfd6_94cc_ab3f;
const TRANSFORM_NONNEGATIVE_Z_BITS: u64 = 0x3f24_b599_aa60_913a;
const REPEAT_NEGATIVE_Z_BITS: u64 = 0xbf04_f8b5_88e3_68f1;
const REPEAT_NONNEGATIVE_Z_BITS: u64 = 0x3f1a_36e2_eb1c_432d;

fn surface() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(-50, -50),
            Point::new(300, -50),
            Point::new(300, 500),
            Point::new(-50, 500),
        ]),
        Vec::new(),
    )
}

fn params(z_bits: u64) -> CrossHatchFillParams {
    CrossHatchFillParams {
        z: f64::from_bits(z_bits),
        spacing: f64::from_bits(0x3f1a_36e2_eb1c_432d),
        overlap: 0.0,
        angle: 0.0,
        density: 1.0,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: f32::from_bits(0x3d4c_cccd),
        dont_sort: true,
    }
}

fn expected(paths: &[&[(i64, i64)]]) -> Vec<Polyline> {
    paths
        .iter()
        .map(|path| Polyline::new(path.iter().map(|&(x, y)| Point::new(x, y)).collect()))
        .collect()
}

#[test]
fn task22o45_public_transform_pattern_preserves_directional_path_order() {
    let surface = surface();
    let negative = fill_surface(
        &surface,
        params(TRANSFORM_NEGATIVE_Z_BITS),
        CoordinateScale::Normal,
    )
    .unwrap();
    let nonnegative = fill_surface(
        &surface,
        params(TRANSFORM_NONNEGATIVE_Z_BITS),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        negative,
        expected(&[&[
            (215, 35),
            (215, 65),
            (185, 135),
            (185, 165),
            (215, 235),
            (215, 265),
            (185, 335),
            (185, 365),
            (215, 435),
            (215, 450),
            (85, 450),
            (85, 435),
            (115, 365),
            (115, 335),
            (85, 265),
            (85, 235),
            (115, 165),
            (115, 135),
            (85, 65),
            (85, 35),
            (100, 0),
            (0, 0),
            (0, 200),
            (15, 235),
            (15, 265),
            (0, 300),
        ]])
    );
    assert_eq!(
        nonnegative,
        expected(&[&[
            (35, 415),
            (65, 415),
            (135, 385),
            (165, 385),
            (235, 415),
            (250, 415),
            (250, 285),
            (235, 285),
            (165, 315),
            (135, 315),
            (65, 285),
            (35, 285),
            (0, 300),
            (35, 215),
            (65, 215),
            (135, 185),
            (165, 185),
            (235, 215),
            (250, 215),
            (250, 85),
            (235, 85),
            (165, 115),
            (135, 115),
            (65, 85),
            (35, 85),
            (0, 100),
        ]])
    );
}

#[test]
fn task22o45_public_repeat_pattern_preserves_directional_path_order() {
    let surface = surface();
    let negative = fill_surface(
        &surface,
        params(REPEAT_NEGATIVE_Z_BITS),
        CoordinateScale::Normal,
    )
    .unwrap();
    let nonnegative = fill_surface(
        &surface,
        params(REPEAT_NONNEGATIVE_Z_BITS),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        negative,
        expected(&[&[(0, 450), (0, 0), (100, 0), (100, 450), (200, 450), (200, 0),]])
    );
    assert_eq!(
        nonnegative,
        expected(&[&[
            (0, 100),
            (250, 100),
            (250, 200),
            (0, 200),
            (0, 300),
            (250, 300),
            (250, 400),
            (0, 400),
        ]])
    );
}
