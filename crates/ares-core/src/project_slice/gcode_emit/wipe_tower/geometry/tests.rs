use super::*;
use crate::GCodeFlavor;

#[test]
fn source_rectangle_wall_closes_at_the_nearest_corner() {
    let scale = CoordinateScale::Normal;
    let bounds = BoxCoordinates::new([1.0, 1.0], [4.0, 4.0]);
    let mut writer = Writer::new(0.2, 0.5, GCodeFlavor::MarlinLegacy, 2.0);
    writer.set_initial_position([5.0, 0.0], [0.0; 2], 0.0);
    writer.set_flow(0.1);
    emit_wall(
        &mut writer,
        &bounds.polygon(scale),
        1200.0,
        (1.0, 1200.0),
        scale,
    );
    assert_eq!(writer.position, [5.0, 1.0]);
    assert!((writer.used_filament - 1.6).abs() < 1e-6);
    assert!(writer.gcode.contains("G1 Y5.000 E0.4000 F1200\n"));
}

#[test]
fn source_scaled_vec2f_divides_in_float_precision() {
    assert_eq!(scaled([0.7, 0.0], CoordinateScale::Normal).x(), 700_000);
}

#[test]
fn source_rib_rounding_uses_the_global_float_sine_overload() {
    // WipeTower2.cpp:143 uses sin(float) from libslic3r.h's <math.h>.
    // Native scalar probe: width 40, depth 24.5, rib width 8, no extension.
    let polygon = Polygon::new(
        [
            (42_089_248, 3_411_018),
            (40_000_000, 4_690_682),
            (40_000_000, 19_809_318),
            (42_089_248, 21_088_982),
            (37_910_752, 27_911_018),
            (32_341_743, 24_500_000),
            (7_658_257, 24_500_000),
            (2_089_248, 27_911_018),
            (-2_089_248, 21_088_982),
            (0, 19_809_318),
            (0, 4_690_682),
            (-2_089_248, 3_411_018),
            (2_089_248, -3_411_018),
            (7_658_257, 0),
            (32_341_743, 0),
            (37_910_752, -3_411_018),
        ]
        .map(|(x, y)| Point::new(x, y))
        .to_vec(),
    );
    let rounded = rounding::round_polygon(&polygon, CoordinateScale::Normal);
    for (index, expected) in [
        (105, Point::new(34_047_252, 25_544_625)),
        (111, Point::new(33_004_095, 25_018_503)),
        (126, Point::new(9_658_256, 24_500_000)),
        (273, Point::new(5_952_748, -1_044_624)),
    ] {
        assert_eq!(rounded.points()[index], expected);
    }
}

#[test]
fn source_cone_scale_uses_the_reduced_width_diagonal() {
    let (radius, stretch) = cone::cone_base(30.0 / 0.66, 50.0, 40.0, 90.0);
    assert!((radius - 50.0).abs() < 1e-12);
    assert!((stretch - 34.0 / 9.0).abs() < 1e-12);
    assert_eq!(cone::cone_base(40.0, 30.0, 10.0, 0.0), (0.0, 1.0));
}
