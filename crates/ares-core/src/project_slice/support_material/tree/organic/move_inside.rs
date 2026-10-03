//! Default-distance `TreeSupport3D.cpp::move_inside_if_outside/move_inside`.

use super::tip_lines::contains;
use crate::geometry::{Coord, Point, Polygon};

pub(super) fn move_inside_if_outside(polygons: &[Polygon], from: Point) -> Point {
    if contains(polygons, from) {
        return from;
    }
    let mut result = from;
    let mut best = f64::MAX;
    for polygon in polygons {
        let points = polygon.points();
        if points.len() < 2 {
            continue;
        }
        let p0 = points[points.len() - 2];
        let mut p1 = points[points.len() - 1];
        let mut beyond_previous = dot(delta(p1, p0), delta(from, p0)) >= squared(delta(p1, p0));
        for &p2 in points {
            let ab = delta(p2, p1);
            let ap = delta(from, p1);
            let length_squared = squared(ab);
            if length_squared <= 0 {
                p1 = p2;
                continue;
            }
            let projection = dot(ab, ap);
            let candidate = if projection <= 0 {
                let candidate = beyond_previous.then_some(p1);
                beyond_previous = false;
                candidate
            } else if projection >= length_squared {
                beyond_previous = true;
                None
            } else {
                beyond_previous = false;
                let fraction = projection as f64 / length_squared as f64;
                Some(Point::new(
                    p1.x() + (ab.0 as f64 * fraction) as Coord,
                    p1.y() + (ab.1 as f64 * fraction) as Coord,
                ))
            };
            p1 = p2;
            let Some(candidate) = candidate else {
                continue;
            };
            let distance = squared(delta(from, candidate)) as f64;
            if distance < best {
                best = distance;
                result = candidate;
            }
        }
    }
    if best < i64::MAX as f64 { result } else { from }
}

fn delta(first: Point, second: Point) -> (Coord, Coord) {
    (first.x() - second.x(), first.y() - second.y())
}

fn dot(first: (Coord, Coord), second: (Coord, Coord)) -> Coord {
    first.0 * second.0 + first.1 * second.1
}

fn squared(vector: (Coord, Coord)) -> Coord {
    dot(vector, vector)
}
