use super::{Params, critical_points, fill_surface, tri_wave};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

#[test]
fn source_wave_and_critical_points_keep_module_symmetry() {
    assert_eq!(tri_wave(0.0, 100.0), 0.0);
    assert_eq!(tri_wave(100.0, 100.0), 0.0);
    assert_eq!(critical_points(0.0, 100.0), [0.0]);
    assert_eq!(
        critical_points(50.0, 100.0),
        [0.0, 25.0, 75.0, 125.0, 175.0]
    );
}

#[test]
fn source_3d_honeycomb_generates_clipped_connected_paths() {
    let square = ExPolygon::new(
        Polygon::new(vec![
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
        ]),
        Vec::new(),
    );
    let output = fill_surface(
        &square,
        Params {
            z: 1.2,
            spacing: 0.407_079_637,
            overlap: 0.0,
            angle: std::f32::consts::FRAC_PI_4,
            density: 0.2,
            multiline: 1,
            anchor_length: 1.0,
            anchor_length_max: 10.0,
            dont_sort: false,
        },
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(!output.is_empty());
    assert!(output.iter().all(|polyline| polyline.is_valid()));
}
