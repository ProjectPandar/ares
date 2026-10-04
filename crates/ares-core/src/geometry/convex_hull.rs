use super::Point;

/// `Geometry/ConvexHull.cpp:11-43`, counter-clockwise from the smallest point.
pub(crate) fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted = points.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let count = sorted.len();
    if count < 3 {
        return Vec::new();
    }
    let ccw = |a: Point, b: Point, c: Point| {
        let cross = (i128::from(b.x()) - i128::from(a.x()))
            * (i128::from(c.y()) - i128::from(a.y()))
            - (i128::from(b.y()) - i128::from(a.y())) * (i128::from(c.x()) - i128::from(a.x()));
        cross > 0
    };
    let mut hull = Vec::with_capacity(2 * count);
    for point in &sorted {
        while hull.len() >= 2 && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    let lower_end = hull.len() + 1;
    for point in sorted[..count - 1].iter().rev() {
        while hull.len() >= lower_end && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    hull.pop();
    hull
}
