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
fn source_cone_scale_uses_the_reduced_width_diagonal() {
    let (radius, stretch) = cone::cone_base(30.0 / 0.66, 50.0, 40.0, 90.0);
    assert!((radius - 50.0).abs() < 1e-12);
    assert!((stretch - 34.0 / 9.0).abs() < 1e-12);
    assert_eq!(cone::cone_base(40.0, 30.0, 10.0, 0.0), (0.0, 1.0));
}
