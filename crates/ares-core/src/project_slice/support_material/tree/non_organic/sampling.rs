//! `generate_contact_points` grid and libnest2d `EdgeCache` sampling.

use crate::geometry::{BoundingBox, Point, Polygon};

pub(super) fn contains(bounds: BoundingBox, point: Point) -> bool {
    point.x() >= bounds.min().x()
        && point.x() <= bounds.max().x()
        && point.y() >= bounds.min().y()
        && point.y() <= bounds.max().y()
}

pub(super) fn radius(bounds: BoundingBox) -> f64 {
    let x = (bounds.max().x() - bounds.min().x()) as f64;
    let y = (bounds.max().y() - bounds.min().y()) as f64;
    0.5 * (x * x + y * y).sqrt()
}

pub(super) fn grid_points(bounds: BoundingBox, sample_step: f64) -> Vec<Point> {
    let angle = 22.0 / 180.0 * std::f64::consts::PI;
    let sin = angle.sin();
    let cos = angle.cos();
    let width = (bounds.max().x() - bounds.min().x()) as f64;
    let depth = (bounds.max().y() - bounds.min().y()) as f64;
    let rotated_x = (width * cos + depth * sin).round() as i64 / 2;
    let rotated_y = (width * sin + depth * cos).round() as i64 / 2;
    let center = bounds.center();
    let mut points = Vec::new();
    let mut x = -rotated_x;
    while x < rotated_x {
        let mut y = -rotated_y;
        while y < rotated_y {
            let point = Point::new(
                (cos * x as f64 - sin * y as f64).round() as i64 + center.x(),
                (cos * y as f64 + sin * x as f64).round() as i64 + center.y(),
            );
            if contains(bounds, point) {
                points.push(point);
            }
            y = (y as f64 + sample_step) as i64;
        }
        x = (x as f64 + sample_step) as i64;
    }
    points
}

pub(super) struct EdgeCache<'a> {
    points: &'a [Point],
    distances: Vec<f64>,
    circumference: f64,
}

impl<'a> EdgeCache<'a> {
    pub(super) fn new(polygon: &'a Polygon) -> Self {
        let points = polygon.points();
        let mut distances = Vec::with_capacity(points.len());
        let mut circumference = 0.0;
        if points.len() > 1 {
            for index in 0..points.len() {
                let first = points[index];
                let second = points[(index + 1) % points.len()];
                let x = (second.x() - first.x()) as f64;
                let y = (second.y() - first.y()) as f64;
                circumference += (x * x + y * y).sqrt();
                distances.push(circumference);
            }
        }
        Self {
            points,
            distances,
            circumference,
        }
    }

    pub(super) fn circumference(&self) -> f64 {
        self.circumference
    }

    pub(super) fn coords(&self, distance: f64) -> Point {
        if self.distances.is_empty() {
            return Point::new(0, 0);
        }
        let distance = distance * self.circumference;
        let index = self.distances.partition_point(|&end| end < distance);
        let first = self.points[index];
        let second = self.points[(index + 1) % self.points.len()];
        let edge_distance = distance
            - if index > 0 {
                self.distances[index - 1]
            } else {
                0.0
            };
        let mut angle = ((second.y() - first.y()) as f64).atan2((second.x() - first.x()) as f64);
        if angle.is_sign_negative() {
            angle += std::f64::consts::TAU;
        }
        Point::new(
            first.x() + (edge_distance * angle.cos()).round() as i64,
            first.y() + (edge_distance * angle.sin()).round() as i64,
        )
    }
}
