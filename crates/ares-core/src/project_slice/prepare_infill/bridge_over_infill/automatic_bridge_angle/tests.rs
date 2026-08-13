mod patterns;
mod reduction;
mod sampling;

use crate::geometry::{Line, Point, Polygon};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

const fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(Point::new(ax, ay), Point::new(bx, by))
}
