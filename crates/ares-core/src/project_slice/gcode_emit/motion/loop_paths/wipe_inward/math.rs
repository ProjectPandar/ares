//! Scalar/vector operations and existing indexed queries used by WipePathHelpers.

use crate::geometry::line_distance_tree::segment::distance_squared;
use crate::geometry::{Line, LineDistanceTree, Point};
use std::ops::{Add, Div, Mul, Sub};

pub(super) const EPSILON: f64 = 1e-4;
pub(super) const MITER_LIMIT: f64 = 3.0;
pub(super) const REVERSAL_DOT: f64 = -0.99;
pub(super) const MIN_ALIGNMENT: f64 = 0.5;

#[derive(Clone, Copy, Default)]
pub(super) struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn from_point(point: Point) -> Self {
        Self {
            x: point.x() as f64,
            y: point.y() as f64,
        }
    }
    pub fn delta(to: Point, from: Point) -> Self {
        Self {
            x: (to.x() - from.x()) as f64,
            y: (to.y() - from.y()) as f64,
        }
    }
    pub fn point(self) -> Point {
        Point::new(self.x as i64, self.y as i64)
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    pub fn squared_norm(self) -> f64 {
        self.dot(self)
    }
    pub fn norm(self) -> f64 {
        self.squared_norm().sqrt()
    }
    pub fn normalized(self) -> Self {
        self / self.norm()
    }
    pub fn left(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}
impl Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Mul<f64> for Vector {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Div<f64> for Vector {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

pub(super) fn tolerance(distance: f64, relative: f64, epsilon: f64) -> f64 {
    (4.0 * epsilon).max(relative * distance)
}

pub(super) fn length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| Vector::delta(pair[1], pair[0]).norm())
        .sum()
}

pub(super) fn lines(points: &[Point]) -> Vec<Line> {
    points
        .windows(2)
        .map(|pair| Line::new(pair[0], pair[1]))
        .collect()
}

pub(super) fn project(line: Line, point: Point) -> (f64, Vector) {
    let (distance, closest) = distance_squared(
        [line.a.x() as f64, line.a.y() as f64],
        [line.b.x() as f64, line.b.y() as f64],
        [point.x() as f64, point.y() as f64],
    );
    (
        distance,
        Vector {
            x: closest[0],
            y: closest[1],
        },
    )
}

pub(super) struct Hit {
    pub distance: f64,
    pub index: usize,
    pub point: Vector,
}

pub(super) struct Distancer<'a> {
    pub lines: &'a [Line],
    index: LineDistanceTree<'a>,
}
impl<'a> Distancer<'a> {
    pub fn new(lines: &'a [Line]) -> Self {
        Self {
            lines,
            index: LineDistanceTree::new(lines),
        }
    }
    pub fn nearest(&self, point: Point) -> Hit {
        let hit = self.index.nearest(point).unwrap();
        Hit {
            distance: hit.squared_distance.sqrt(),
            index: hit.line_index,
            point: project(self.lines[hit.line_index], point).1,
        }
    }
    pub fn radius(&self, point: Point, radius: f64) -> Vec<usize> {
        self.index.all_lines_in_radius(point, radius)
    }
    pub fn intersections(&self, line: Line) -> Vec<(Point, usize)> {
        // The caller rejects any nonlocal contact, independently of hit order.
        self.index.intersections_sorted(line)
    }
}

pub(super) fn clip_end(points: &mut Vec<Point>, mut distance: f64) {
    while distance > 0.0 {
        let Some(last) = points.pop() else { return };
        let Some(&previous) = points.last() else {
            return;
        };
        let vector = Vector::delta(previous, last);
        let squared = vector.squared_norm();
        if squared > distance * distance {
            points.push((Vector::from_point(last) + vector * (distance / squared.sqrt())).point());
            break;
        }
        distance -= squared.sqrt();
    }
}

pub(super) fn sample(points: &[Point], mut target: f64, epsilon: f64) -> Point {
    let mut result = points[0];
    for &point in points.iter().skip(1) {
        if target <= 0.0 {
            break;
        }
        let segment = Vector::delta(point, result);
        let distance = segment.norm();
        if distance <= epsilon {
            continue;
        }
        if target <= distance {
            return (Vector::from_point(result) + segment * (target / distance)).point();
        }
        target -= distance;
        result = point;
    }
    result
}
