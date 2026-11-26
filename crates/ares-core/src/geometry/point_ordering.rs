//! Point ordering strategies: ShortestPath.cpp:1624–1650 and
//! GCode/OrderingStrategies.{hpp,cpp} at
//! OrcaSlicer 40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19.

mod crossing_grid;
mod crossings;
mod exact_cycle;
mod nearest_points;
mod or_opt;
mod snake;
mod two_opt;

use super::{CoordinateScale, Point};

pub(crate) fn order_points_with_strategy(
    points: &[Point],
    print_order: crate::ProcessPrintOrder,
    start_near: Option<Point>,
    scale: CoordinateScale,
) -> Vec<usize> {
    if points.is_empty() {
        return Vec::new();
    }
    match print_order {
        crate::ProcessPrintOrder::Default | crate::ProcessPrintOrder::AsObjectList => {
            chain_points_with_postprocessing(points, start_near, scale)
        }
        crate::ProcessPrintOrder::Snake => snake_with_start(points, start_near),
        crate::ProcessPrintOrder::BestOfStrategies => {
            let default_path = chain_points_with_postprocessing(points, start_near, scale);
            let snake_path = snake_with_start(points, start_near);
            pick_shorter_cycle(default_path, snake_path, points)
        }
    }
}

/// OrderingStrategies.hpp:71–122 and OrderingStrategies.cpp:440–448.
pub(crate) fn chain_instances_snake(points: &[Point], start_near: Option<Point>) -> Vec<usize> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut centers = points.to_vec();
    let mut indices = (0..points.len()).collect::<Vec<_>>();
    if let Some(start) = start_near {
        let best_start = closest_point_index(&centers, start);
        centers.rotate_left(best_start);
        indices.rotate_left(best_start);
    }
    let mut path = snake::snake_core(&centers);
    if start_near.is_some() && !path.is_empty() {
        if let Some(pos) = path.iter().position(|&idx| idx == 0)
            && pos != 0
        {
            path.rotate_left(pos);
        }
    } else {
        rotate_minimize_closing(&mut path, &centers);
    }
    path.into_iter().map(|step| indices[step]).collect()
}

/// OrderingStrategies.cpp:454–495.
pub(crate) fn chain_instances_best_of(
    points: &[Point],
    start_near: Option<Point>,
    scale: CoordinateScale,
) -> Vec<usize> {
    if points.is_empty() {
        return Vec::new();
    }
    let default_path = chain_points_with_postprocessing(points, start_near, scale);
    let snake_path = chain_instances_snake(points, start_near);
    pick_shorter_cycle(default_path, snake_path, points)
}

fn snake_with_start(points: &[Point], start_near: Option<Point>) -> Vec<usize> {
    let mut path = snake::snake_core(points);
    if let Some(start) = start_near
        && !path.is_empty()
    {
        let best_start = closest_point_index(points, start);
        if let Some(pos) = path.iter().position(|&idx| idx == best_start)
            && pos != 0
        {
            path.rotate_left(pos);
        }
    } else {
        rotate_minimize_closing(&mut path, points);
    }
    path
}

fn closest_point_index(points: &[Point], start: Point) -> usize {
    let start_pos = position(start);
    let mut best_start = 0;
    let mut best_d2 = f64::MAX;
    for (index, &point) in points.iter().enumerate() {
        let d2 = squared(position(point), start_pos);
        if d2 < best_d2 {
            best_d2 = d2;
            best_start = index;
        }
    }
    best_start
}

fn pick_shorter_cycle(first: Vec<usize>, second: Vec<usize>, points: &[Point]) -> Vec<usize> {
    let first_len = cycle_path_length(&first, points);
    let first_edge = max_edge_length(&first, points);
    let second_len = cycle_path_length(&second, points);
    let second_edge = max_edge_length(&second, points);
    if second_len < first_len || (second_len == first_len && second_edge < first_edge) {
        second
    } else {
        first
    }
}

fn cycle_path_length(path: &[usize], points: &[Point]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for index in 0..path.len() {
        let next = (index + 1) % path.len();
        total += distance(points[path[index]], points[path[next]]);
    }
    total
}

fn max_edge_length(path: &[usize], points: &[Point]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }
    let mut max_edge = 0.0;
    for index in 0..path.len() {
        let next = (index + 1) % path.len();
        let edge = distance(points[path[index]], points[path[next]]);
        if edge > max_edge {
            max_edge = edge;
        }
    }
    max_edge
}

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
