use super::super::{PlanePathFillParams, PlanePathPattern, fill_surface};
use crate::geometry::{BoundingBox, CoordinateScale, ExPolygon, Point, Polygon};

#[test]
fn each_plane_path_clips_repeatably_to_the_solid_surface() {
    let surface = square(2_000_000);
    let bounds = BoundingBox::from_expolygon(&surface).unwrap();
    let params = PlanePathFillParams {
        spacing: 1.0,
        overlap: 0.5,
        density: 1.0,
        angle: -std::f32::consts::FRAC_PI_2,
        multiline: 1,
        resolution: 0.0125,
        anchor_length: 0.0,
        anchor_length_max: 20.0,
        object_bounding_box: bounds,
        calibration_order: false,
    };

    for pattern in [
        PlanePathPattern::HilbertCurve,
        PlanePathPattern::ArchimedeanChords,
        PlanePathPattern::OctagramSpiral,
    ] {
        let first = fill_surface(&surface, pattern, params, CoordinateScale::Normal).unwrap();
        let second = fill_surface(&surface, pattern, params, CoordinateScale::Normal).unwrap();

        assert_eq!(first, second, "{pattern:?}");
        assert!(!first.is_empty(), "{pattern:?}");
        assert!(
            first.iter().flatten().all(|polyline| polyline.is_valid()
                && polyline.points().iter().all(|point| {
                    (-2_000_000..=2_000_000).contains(&point.x())
                        && (-2_000_000..=2_000_000).contains(&point.y())
                })),
            "{pattern:?}"
        );
    }
}

fn square(radius: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(-radius, -radius),
            Point::new(radius, -radius),
            Point::new(radius, radius),
            Point::new(-radius, radius),
        ]),
        Vec::new(),
    )
}
