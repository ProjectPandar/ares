//! Loop center placement and the existence query of Point.hpp's radius lookup.

use crate::geometry::{CoordinateScale, Point, Polygon};
use std::collections::HashMap;

pub(super) struct CircleCenters {
    distance: f64,
    radius_squared: f64,
    shift: u32,
    resolution: i64,
    grid: HashMap<(i64, i64), Vec<Point>>,
    points: Vec<Point>,
    last: Point,
}

impl CircleCenters {
    pub(super) fn new(distance: f64, scale: CoordinateScale) -> Self {
        let radius = (distance - 1e-4 / scale.factor()) as i64;
        let gridres = 2 * radius + 4;
        let shift = 64 - (gridres as u64).leading_zeros();
        Self {
            distance,
            radius_squared: (radius as f64) * (radius as f64),
            shift,
            resolution: 1 << shift,
            grid: HashMap::new(),
            points: Vec::new(),
            last: Point::new(0, 0),
        }
    }

    pub(super) fn points(&self) -> &[Point] {
        &self.points
    }

    fn insert(&mut self) {
        let key = (self.last.x() >> self.shift, self.last.y() >> self.shift);
        self.grid.entry(key).or_default().push(self.last);
        self.points.push(self.last);
    }

    fn too_close(&self) -> bool {
        let x = (self.last.x() + (self.resolution >> 1)) >> self.shift;
        let y = (self.last.y() + (self.resolution >> 1)) >> self.shift;
        [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)]
            .iter()
            .filter_map(|key| self.grid.get(key))
            .flatten()
            .any(|point| {
                let vx = (self.last.x() - point.x()) as f64;
                let vy = (self.last.y() - point.y()) as f64;
                vx * vx + vy * vy < self.radius_squared
            })
    }

    pub(super) fn walk(&mut self, contour: &Polygon) {
        let points = contour.points();
        let mut position = WalkPosition {
            segment: None,
            t: 0.0,
        };
        if self.points.is_empty() {
            position.segment = Some(0);
            self.last = points[0];
            self.insert();
        }
        let mut index = 1;
        while index < points.len() {
            let Some((point, check_proximity)) =
                next_candidate(points, index, self.last, self.distance, &mut position)
            else {
                index += 1;
                continue;
            };
            self.last = point;
            if check_proximity && self.too_close() {
                continue;
            }
            self.insert();
            index += 1;
        }
    }
}

struct WalkPosition {
    segment: Option<usize>,
    t: f64,
}

fn next_candidate(
    points: &[Point],
    index: usize,
    last: Point,
    distance: f64,
    position: &mut WalkPosition,
) -> Option<(Point, bool)> {
    let p1 = points[index - 1];
    let p2 = points[index];
    let vx = p2.x() as f64 - p1.x() as f64;
    let vy = p2.y() as f64 - p1.y() as f64;
    let cx = (p1.x() - last.x()) as f64;
    let cy = (p1.y() - last.y()) as f64;
    let a = vx * vx + vy * vy;
    let b = 2.0 * (vx * cx + vy * cy);
    let c = cx * cx + cy * cy - distance * distance;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant <= 0.0 {
        position.segment = Some(index - 1);
        position.t = 0.0;
        return Some((p1, false));
    }
    let root = discriminant.sqrt();
    let t1 = (-b - root) / (2.0 * a);
    let t2 = (-b + root) / (2.0 * a);
    let t0 = if position.segment == Some(index - 1) {
        position.t
    } else {
        0.0
    };
    let t = if t0 <= t1 {
        t0
    } else if t2 <= 1.0 {
        t2
    } else {
        position.segment = None;
        return None;
    };
    position.segment = Some(index - 1);
    position.t = t;
    Some((
        Point::new(p1.x() + (vx * t) as i64, p1.y() + (vy * t) as i64),
        true,
    ))
}
