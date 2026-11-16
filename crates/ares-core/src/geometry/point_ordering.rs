//! Default tours: ShortestPath.cpp:1624–1650 at
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

mod crossing_grid;
mod crossings;
mod exact_cycle;
mod nearest_points;
mod or_opt;
mod two_opt;

use super::{CoordinateScale, Point};

pub(crate) fn chain_points_with_postprocessing(
    points: &[Point],
    start_near: Option<Point>,
    scale: CoordinateScale,
) -> Vec<usize> {
    let mut path = if points.len() <= 10 {
        let mut path = exact_cycle::tsp_exact_cycle(points);
        if let Some(start) = start_near
            && path.len() > 2
        {
            rotate_near(&mut path, points, start);
        }
        path
    } else {
        let mut path = super::chain_points::chain_points_with_start(points, start_near);
        for _ in 0..3 {
            let mut improved = two_opt::improve(&mut path, points);
            improved |= or_opt::improve(&mut path, points, scale.scaled_delta(0.0001));
            improved |= crossings::remove(&mut path, points);
            if !improved {
                break;
            }
        }
        path
    };
    if start_near.is_none() {
        rotate_minimize_closing(&mut path, points);
    }
    path
}

// ShortestPath.cpp:1630–1634 subtracts integer points before casting, unlike
// OrderingStrategies.cpp's cast-before-subtraction cycle metrics.
fn difference_squared(a: Point, b: Point) -> f64 {
    let dx = (a.x() - b.x()) as f64;
    let dy = (a.y() - b.y()) as f64;
    dx * dx + dy * dy
}

fn rotate_near(path: &mut [usize], points: &[Point], start: Point) {
    let mut best = 0;
    for index in 1..path.len() {
        if difference_squared(points[path[index]], start)
            < difference_squared(points[path[best]], start)
        {
            best = index;
        }
    }
    path.rotate_left(best);
    if difference_squared(points[path[path.len() - 1]], points[path[0]])
        < difference_squared(points[path[1]], points[path[0]])
    {
        path[1..].reverse();
    }
}

// OrderingStrategies.cpp:268–280::tsp_rotate_minimize_closing.
fn rotate_minimize_closing(path: &mut [usize], points: &[Point]) {
    let mut best = 0;
    let mut best_distance = f64::MAX;
    for start in 0..path.len() {
        let last = (start + path.len() - 1) % path.len();
        let distance = squared(position(points[path[start]]), position(points[path[last]]));
        if distance < best_distance {
            best_distance = distance;
            best = start;
        }
    }
    path.rotate_left(best);
}

fn position(point: Point) -> [f64; 2] {
    [point.x() as f64, point.y() as f64]
}

fn squared(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

fn distance(a: Point, b: Point) -> f64 {
    squared(position(a), position(b)).sqrt()
}
