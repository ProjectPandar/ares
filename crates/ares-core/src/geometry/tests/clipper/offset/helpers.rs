use crate::geometry::clipper::{ClipperOffset, JoinType};
use crate::geometry::{Point, Polygon};

pub(super) fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

pub(super) fn coordinates(path: &Polygon) -> Vec<(i64, i64)> {
    path.points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

pub(super) fn raw(points: &[(i64, i64)], join: JoinType, delta: f64) -> Vec<Vec<(i64, i64)>> {
    let mut offset = ClipperOffset::default();
    offset.add_closed_path(&polygon(points), join);
    offset.generate_raw(delta).iter().map(coordinates).collect()
}

#[test]
fn task22g_offset_defaults_match_fixed_clipper() {
    let offset = ClipperOffset::default();

    assert_eq!(offset.miter_limit(), 2.0);
    assert_eq!(offset.arc_tolerance(), 0.25);
    assert_eq!(offset.shortest_edge_length(), 0.0);
}

#[test]
fn task22g_generated_coordinates_use_fixed_round_at_f64_boundaries() {
    let input = [(-10, 0), (0, 0), (0, 10), (-10, 10)];

    assert_eq!(
        raw(&input, JoinType::Miter, 0.499_999_999_999_999_94,),
        vec![vec![(-10, 0), (0, 0), (0, 11), (-10, 11)]]
    );
    assert_eq!(
        raw(&input, JoinType::Miter, 0.5),
        vec![vec![(-10, 0), (1, 0), (1, 11), (-10, 11)]]
    );
}
