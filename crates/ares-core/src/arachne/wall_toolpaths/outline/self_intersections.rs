use crate::geometry::{CoordinateScale, Point, Polygon};

use super::line_grid::LineGrid;

/// WallToolPaths.cpp:282–326::fixSelfIntersections, including near intersections.
pub(super) fn fix(polygons: &mut [Polygon], epsilon: i64, scale: CoordinateScale) {
    if epsilon < 1 {
        return;
    }
    let grid = LineGrid::new(polygons, scale.checked_scale(2.0).unwrap());
    for polygon_index in 0..polygons.len() {
        for point_index in 0..polygons[polygon_index].points().len() {
            fix_point(polygons, polygon_index, point_index, epsilon, &grid);
        }
    }
}

fn fix_point(
    polygons: &mut [Polygon],
    polygon_index: usize,
    point_index: usize,
    epsilon: i64,
    grid: &LineGrid,
) {
    let half_epsilon = (epsilon + 1) / 2;
    let move_distance = 2.max(half_epsilon - 2);
    let count = polygons[polygon_index].points().len();
    let point = polygons[polygon_index].points()[point_index];
    for (line_polygon, line_index) in grid.nearby(point, epsilon) {
        let line_points = polygons[line_polygon].points();
        let next_line_index = (line_index + 1) % line_points.len();
        if polygon_index == line_polygon
            && (point_index == line_index || point_index == next_line_index)
        {
            continue;
        }
        let (a, b) = (line_points[line_index], line_points[next_line_index]);
        let point = polygons[polygon_index].points()[point_index];
        let closest = closest_point(point, a, b);
        let dx = point.x() - closest.x();
        let dy = point.y() - closest.y();
        if half_epsilon * half_epsilon >= dx * dx + dy * dy {
            let other = polygons[polygon_index].points()[(point_index + 1) % count];
            let mut vx = b.x() - a.x();
            let mut vy = b.y() - a.y();
            if vx * (other.y() - a.y()) - vy * (other.x() - a.x()) <= 0 {
                vx = -vx;
                vy = -vy;
            }
            let length = ((vx * vx + vy * vy) as f64).sqrt() as i64;
            polygons[polygon_index].points_mut()[point_index] = Point::new(
                point.x() + (-vy * move_distance) / length,
                point.y() + (vx * move_distance) / length,
            );
        }
    }
}

// Line.hpp:44–71 returns a truncating integer projection, not a rounded point.
fn closest_point(point: Point, a: Point, b: Point) -> Point {
    let vx = (b.x() - a.x()) as f64;
    let vy = (b.y() - a.y()) as f64;
    let ax = (point.x() - a.x()) as f64;
    let ay = (point.y() - a.y()) as f64;
    let length_squared = vx * vx + vy * vy;
    if length_squared == 0.0 {
        return a;
    }
    let t = (ax * vx + ay * vy) / length_squared;
    if t <= 0.0 {
        a
    } else if t >= 1.0 {
        b
    } else {
        Point::new(
            (a.x() as f64 + t * vx) as i64,
            (a.y() as f64 + t * vy) as i64,
        )
    }
}
