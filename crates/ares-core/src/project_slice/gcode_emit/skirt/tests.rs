use super::{convex_hull, find_start_point, split_at_nearest};
use crate::geometry::Point;

/// `Geometry/ConvexHull.cpp:11-43`: sorted monotone chain, collinear points
/// dropped, counter-clockwise result starting at the smallest point.
#[test]
fn convex_hull_matches_upstream_ordering() {
    let points = [
        Point::new(0, 0),
        Point::new(1000, 0),
        Point::new(1000, 1000),
        Point::new(0, 1000),
        Point::new(500, 500),
        Point::new(500, 0),
    ];
    let hull = convex_hull(&points);
    assert_eq!(
        hull,
        vec![
            Point::new(0, 0),
            Point::new(1000, 0),
            Point::new(1000, 1000),
            Point::new(0, 1000),
        ]
    );
}

#[test]
fn convex_hull_drops_degenerate_input() {
    assert!(convex_hull(&[Point::new(1, 1)]).is_empty());
    assert!(convex_hull(&[]).is_empty());
}

/// `GCode.cpp:4334-4359`: target sits on the bounding-circle radius at the
/// configured angle around the bbox center.
#[test]
fn find_start_point_uses_angle_from_center() {
    let points = [
        Point::new(-1000, -1000),
        Point::new(1000, -1000),
        Point::new(1000, 1000),
        Point::new(-1000, 1000),
    ];
    // -135deg: down-left of center; radius sqrt(2)*1000 times cos/sin
    // (-0.7071) lands on (-1000,-1000) (truncated).
    let target = find_start_point(&points, -135.0);
    assert_eq!(target, Point::new(-1000, -1000));
    let target = find_start_point(&points, 0.0);
    assert_eq!(target, Point::new(1414, 0));
}

/// `GCode.cpp:5775` split at the nearest point: the loop projects onto
/// edges (`Point.cpp:106`), rotates to the seam, and repeats it so the
/// loop closes.
#[test]
fn split_at_nearest_projects_rotates_and_closes() {
    let points = [
        Point::new(0, 0),
        Point::new(100, 0),
        Point::new(100, 100),
        Point::new(0, 100),
    ];
    // Nearest point to (105, 95) is the projection onto the right edge.
    let split = split_at_nearest(&points, Point::new(105, 95));
    assert_eq!(
        split,
        vec![
            Point::new(100, 95),
            Point::new(100, 100),
            Point::new(0, 100),
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 95),
        ]
    );
    // A target past an endpoint clamps to that endpoint and rotates there
    // without duplicating it.
    let split = split_at_nearest(&points, Point::new(105, -5));
    assert_eq!(
        split,
        vec![
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
            Point::new(0, 0),
            Point::new(100, 0),
        ]
    );
}
