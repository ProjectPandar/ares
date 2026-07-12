use crate::Point2;

pub(super) type Bounds = (f64, f64, f64, f64);

pub(super) fn bounds(points: &[Point2]) -> Option<Bounds> {
    if points.len() != 4 {
        return None;
    }
    let min_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    let mut corners = [
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ];
    corners.sort_by(compare_points);
    let mut actual = [points[0], points[1], points[2], points[3]];
    actual.sort_by(compare_points);
    if actual == corners {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

fn compare_points(left: &Point2, right: &Point2) -> std::cmp::Ordering {
    left.x()
        .total_cmp(&right.x())
        .then_with(|| left.y().total_cmp(&right.y()))
}
