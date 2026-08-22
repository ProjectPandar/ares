mod annotate;
mod chaining;
mod diagram;
mod eligibility;
mod error;
mod postprocess;
mod validate;

use super::super::{
    CoordinateScale, ExPolygon, Point, Polygon, medial_axis, medial_axis::MedialAxisError,
    medial_axis::validate::integer_point,
};

#[test]
fn task22o13_empty_expolygon_has_no_medial_axis() {
    let input = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
    assert_eq!(
        medial_axis(&input, 1.0, 10.0, CoordinateScale::Normal),
        Ok(Vec::new())
    );
}

#[test]
fn task22o13_rectangle_has_literal_center_axis_width_and_direction() {
    let input = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000, 0),
            Point::new(1_000, 400),
            Point::new(0, 400),
        ]),
        Vec::new(),
    );
    let normal = medial_axis(&input, 0.0, 1_000.0, CoordinateScale::Normal).unwrap();
    let large_bed = medial_axis(&input, 0.0, 1_000.0, CoordinateScale::LargeBed).unwrap();
    let expected = vec![super::super::ThickPolyline {
        points: vec![Point::new(1_000, 200), Point::new(0, 200)],
        width: vec![400.0, 400.0],
        endpoints: (true, true),
    }];
    assert_eq!(normal, expected);
    assert_eq!(large_bed, expected);
}

#[test]
fn task22o13_repeated_adjacent_point_is_a_typed_construction_error() {
    let input = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 0),
            Point::new(0, 10),
        ]),
        Vec::new(),
    );
    assert_eq!(
        medial_axis(&input, 1.0, 10.0, CoordinateScale::Normal),
        Err(MedialAxisError::ConstructionFailed)
    );
}

#[test]
fn task22o209_voronoi_vertices_round_half_away_from_zero() {
    assert_eq!(integer_point(7.9, -7.9), Point::new(8, -8));
    assert_eq!(integer_point(0.9, -0.9), Point::new(1, -1));
}
