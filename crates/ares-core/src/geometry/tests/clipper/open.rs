mod horizontals;
mod input;
mod large_coordinates;
mod operations;
mod polygon_overloads;
mod polytree;
mod recombine;

use crate::geometry::{Point, Polygon, Polyline};

fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| point(x, y)).collect())
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| point(x, y)).collect())
}

fn square() -> Polygon {
    polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)])
}

fn coordinates(paths: &[Polyline]) -> Vec<Vec<(i64, i64)>> {
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
