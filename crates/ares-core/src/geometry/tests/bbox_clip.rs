use crate::geometry::{
    BoundingBox, ExPolygon, Point, Polygon, clip_clipper_expolygons_with_subject_bbox,
    clip_clipper_polygons_with_subject_bbox,
};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn bounds() -> BoundingBox {
    BoundingBox::from_polygon(&polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)])).unwrap()
}

#[test]
fn task22o2_bbox_clip_discards_three_neighbor_runs_on_one_outside_side() {
    let input = polygon(&[(-5, 2), (-4, 4), (-5, 6), (2, 5)]);
    assert_eq!(
        clip_clipper_polygons_with_subject_bbox(&[input], bounds()),
        vec![polygon(&[(-5, 2), (-5, 6), (2, 5)])]
    );
}

#[test]
fn task22o2_bbox_clip_discards_wholly_outside_and_preserves_inside_order() {
    let outside = polygon(&[(-9, -9), (-8, -9), (-8, -8), (-9, -8)]);
    let inside = polygon(&[(1, 1), (9, 1), (9, 9), (1, 9)]);
    assert_eq!(
        clip_clipper_polygons_with_subject_bbox(&[outside, inside.clone()], bounds()),
        vec![inside]
    );
}

#[test]
fn task22o2_bbox_clip_flattens_contour_then_holes_and_omits_degenerate_paths() {
    let contour = polygon(&[(-2, 5), (5, -2), (12, 5), (5, 12)]);
    let hole = polygon(&[(2, 2), (2, 8), (8, 8), (8, 2)]);
    let value = ExPolygon::new(
        contour.clone(),
        vec![hole.clone(), polygon(&[(1, 1), (2, 2)])],
    );
    assert_eq!(
        clip_clipper_expolygons_with_subject_bbox(&[value], bounds()),
        vec![contour, hole]
    );
}

#[test]
fn task22o2_bbox_inflation_uses_fixed_coordinates_at_negative_values() {
    let mut value = BoundingBox::from_polygon(&polygon(&[(-10, -8), (-2, 4), (1, 2)])).unwrap();
    value.offset(10);
    assert_eq!(value.min(), Point::new(-20, -18));
    assert_eq!(value.max(), Point::new(11, 14));
}
