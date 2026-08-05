mod closing;
mod volumes;

use crate::geometry::{ExPolygon, Point, Polygon};

use super::super::filter::flatten_expolygons;
use super::rectangle;

fn coordinates(paths: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    paths
        .iter()
        .map(|path| {
            path.points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

#[test]
fn task22o23_neighbor_flattening_is_expolygon_contour_then_holes_in_collection_order() {
    let first_contour = rectangle(0, 0, 100, 100);
    let first_hole = Polygon::new(vec![
        Point::new(20, 20),
        Point::new(20, 40),
        Point::new(40, 40),
        Point::new(40, 20),
    ]);
    let second_contour = rectangle(200, 0, 300, 100);
    let flattened = flatten_expolygons(&[
        ExPolygon::new(first_contour.clone(), vec![first_hole.clone()]),
        ExPolygon::new(second_contour.clone(), Vec::new()),
    ]);
    assert_eq!(flattened, vec![first_contour, first_hole, second_contour]);
}
