use crate::geometry::{
    BoundingBox, ExPolygon, Point, Polygon, chain_expolygons, chain_expolygons_order,
};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn expolygon(points: &[(i64, i64)], holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(polygon(points), holes)
}

#[test]
fn task22o1_bbox_ordering_handles_negative_coordinates_and_truncated_centers() {
    let bounds =
        BoundingBox::from_polygon(&polygon(&[(-9, -7), (-2, -7), (-2, 4), (-9, 4)])).unwrap();
    assert_eq!(bounds.center(), Point::new(-5, -1));
}

#[test]
fn task22o1_bbox_ordering_uses_only_the_expolygon_contour() {
    let value = expolygon(
        &[(0, 0), (20, 0), (20, 10), (0, 10)],
        vec![polygon(&[(-100, -100), (100, -100), (100, 100)])],
    );
    assert_eq!(
        BoundingBox::from_expolygon(&value).unwrap().center(),
        Point::new(10, 5)
    );
}

#[test]
fn task22o1_bbox_ordering_is_nearest_neighbor_and_repeatable() {
    let values = vec![
        expolygon(&[(100, 0), (110, 0), (110, 10), (100, 10)], vec![]),
        expolygon(&[(0, 0), (10, 0), (10, 10), (0, 10)], vec![]),
        expolygon(&[(20, 0), (30, 0), (30, 10), (20, 10)], vec![]),
    ];
    let first = chain_expolygons_order(&values);
    assert_eq!(first, vec![0, 2, 1]);
    assert_eq!(first, chain_expolygons_order(&values));
    assert_eq!(first.len(), values.len());
    assert_eq!(chain_expolygons(Vec::new()), Vec::new());
}

#[test]
fn task22o1_bbox_ordering_preserves_each_input_for_equal_centers() {
    let values = vec![
        expolygon(&[(0, 0), (10, 0), (10, 10), (0, 10)], vec![]),
        expolygon(&[(1, 1), (9, 1), (9, 9), (1, 9)], vec![]),
    ];
    assert_eq!(chain_expolygons_order(&values), vec![0, 1]);
}
