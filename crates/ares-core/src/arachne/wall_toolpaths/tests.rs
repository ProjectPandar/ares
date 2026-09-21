use crate::geometry::{CoordinateScale, Point, Polygon};

use super::{RawWallToolPathConfig, generate, generate_raw, outline};

fn fixture() -> (Polygon, RawWallToolPathConfig) {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    (
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(scaled(10.0), 0),
            Point::new(scaled(10.0), scaled(10.0)),
            Point::new(0, scaled(10.0)),
        ]),
        RawWallToolPathConfig {
            outer_spacing: scaled(0.4),
            inner_spacing: scaled(0.4),
            inset_count: 26,
            outer_wall_inset: 0,
            layer_height: scaled(0.2),
            min_bead_width: scaled(0.34),
            min_feature_size: scaled(0.1),
            transition_length: scaled(0.4),
            transitioning_angle: f64::from((std::f32::consts::PI * 10.0_f32) / 180.0_f32),
            transition_filter_deviation: scaled(0.1),
            wall_distribution_count: 1,
            coordinate_scale: scale,
            min_length_factor: 0.5,
            wall_maximum_resolution: scaled(0.5),
            wall_maximum_deviation: scaled(0.025),
            is_top_or_bottom_layer: false,
        },
    )
}

#[test]
fn task22o195_rectangle_generates_positive_width_raw_wall_toolpaths() {
    let (outline, config) = fixture();

    let toolpaths = generate_raw(&[outline], config).unwrap();

    assert!(toolpaths.iter().flatten().any(|line| {
        line.inset_index < config.inset_count
            && line.junctions.len() >= 2
            && line.junctions.iter().all(|junction| junction.width > 0)
    }));
}

#[test]
fn task22o196_wall_toolpath_pipeline_stitches_closed_rectangle_lines() {
    let (outline, config) = fixture();
    let prepared = outline::prepare(std::slice::from_ref(&outline), config).unwrap();
    assert!(!prepared.is_empty());

    let generated = generate(&[outline], config).unwrap();

    assert!(
        generated
            .toolpaths
            .iter()
            .flatten()
            .any(|line| line.is_closed)
    );
    assert!(
        generated
            .toolpaths
            .iter()
            .flatten()
            .flat_map(|line| &line.junctions)
            .all(|junction| junction.width > 0)
    );
}

#[test]
fn task22o203_prepares_fixture_domain_before_trapezoidation() {
    let (_, mut config) = fixture();
    config.outer_spacing = 377_079;
    config.inner_spacing = 377_079;
    config.inset_count = 29;
    let outline = Polygon::new(vec![
        Point::new(2_690_706, -20_263_054),
        Point::new(-1_602_493, -15_969_855),
        Point::new(-1_469_177, -15_836_537),
        Point::new(-862_793, -15_909_651),
        Point::new(-831_312, -15_909_411),
        Point::new(-1_519_521, -15_813_200),
        Point::new(-2_495_958, -15_601_912),
        Point::new(-3_040_178, -15_548_538),
        Point::new(-7_889_784, -15_548_538),
        Point::new(-5_249_484, -18_188_838),
        Point::new(-4_583_039, -17_856_194),
        Point::new(-3_765_477, -17_595_579),
        Point::new(-2_926_077, -17_465_343),
        Point::new(-2_073_923, -17_465_343),
        Point::new(-1_234_523, -17_595_579),
        Point::new(-416_961, -17_856_194),
        Point::new(347_353, -18_237_688),
        Point::new(1_041_851, -18_730_630),
        Point::new(1_480_065, -19_137_235),
        Point::new(1_657_323, -19_329_792),
        Point::new(2_027_204, -19_793_598),
        Point::new(2_170_378, -20_012_730),
        Point::new(2_499_256, -20_582_360),
    ]);

    let generated = generate(&[outline], config).unwrap();

    assert!(
        generated
            .toolpaths
            .iter()
            .flatten()
            .flat_map(|line| &line.junctions)
            .all(|junction| junction.width > 0)
    );
}
