use crate::geometry::{
    CoordinateScale, ExPolygon, Point, Polygon, medial_axis, medial_axis::MedialAxisError,
};

#[test]
fn task22o13_zero_length_closing_site_is_a_typed_error() {
    let input = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(0, 10),
            Point::new(0, 0),
        ]),
        Vec::new(),
    );
    assert_eq!(
        medial_axis(&input, 1.0, 10.0, CoordinateScale::Normal),
        Err(MedialAxisError::ConstructionFailed)
    );
}
