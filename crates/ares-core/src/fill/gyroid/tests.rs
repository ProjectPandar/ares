use super::{GyroidFillParams, fill_surface};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

fn square() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(-10_000_000, -10_000_000),
            Point::new(10_000_000, -10_000_000),
            Point::new(10_000_000, 10_000_000),
            Point::new(-10_000_000, 10_000_000),
        ]),
        Vec::new(),
    )
}

fn params(z: f64) -> GyroidFillParams {
    GyroidFillParams {
        z,
        spacing: 0.45,
        overlap: 0.0,
        angle: std::f32::consts::FRAC_PI_4,
        density: 0.15,
        multiline: 1,
        anchor_length: 1.5,
        anchor_length_max: 20.0,
        dont_sort: false,
    }
}

#[test]
fn gyroid_generates_connected_paths_inside_surface() {
    let paths = fill_surface(&square(), params(1.0), CoordinateScale::Normal).unwrap();

    assert!(!paths.is_empty());
    assert!(paths.iter().all(|path| path.points().len() >= 2));
    assert!(paths.iter().flat_map(|path| path.points()).all(|point| {
        (-10_000_000..=10_000_000).contains(&point.x())
            && (-10_000_000..=10_000_000).contains(&point.y())
    }));
}

#[test]
fn gyroid_phase_changes_with_print_z() {
    let first = fill_surface(&square(), params(1.0), CoordinateScale::Normal).unwrap();
    let second = fill_surface(&square(), params(1.2), CoordinateScale::Normal).unwrap();

    assert_ne!(first, second);
}
