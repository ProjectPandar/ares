//! libnest2d's `EdgeCache` (`placers/nfpplacer.hpp:145-300` at `345efec64`): a no-fit polygon's
//! outline by cumulative length, sampled at evenly strided corners for the optimizer starts.

use crate::geometry::{ExPolygon, Point};

struct Contour {
    corners: Vec<f64>,
    edges: Vec<(Point, Point)>,
    distances: Vec<f64>,
    full_distance: f64,
}

impl Contour {
    fn new(points: &[Point]) -> Self {
        let mut contour = Self {
            corners: Vec::new(),
            edges: Vec::new(),
            distances: Vec::with_capacity(points.len()),
            full_distance: 0.0,
        };
        for pair in points.windows(2) {
            contour.edges.push((pair[0], pair[1]));
            let (dx, dy) = (
                pair[1].x() as f64 - pair[0].x() as f64,
                pair[1].y() as f64 - pair[0].y() as f64,
            );
            contour.full_distance += (dx * dx + dy * dy).sqrt();
            contour.distances.push(contour.full_distance);
        }
        contour
    }

    /// The point at `distance` (0..1) along the outline (`coords`).
    fn coords(&self, distance: f64) -> Point {
        if self.distances.is_empty() || self.edges.is_empty() {
            return Point::new(0, 0);
        }
        let distance = if distance > 1.0 {
            distance % 1.0
        } else {
            distance
        };
        let d = distance * self.full_distance;
        let index = self.distances.partition_point(|value| *value < d);
        let (first, second) = self.edges[index];
        let along = d - if index > 0 {
            self.distances[index - 1]
        } else {
            0.0
        };
        // `_Segment::angleToXaxis`, shifted into [0, 2π).
        let mut angle = ((second.y() - first.y()) as f64).atan2((second.x() - first.x()) as f64);
        if angle.is_sign_negative() {
            angle += 2.0 * std::f64::consts::PI;
        }
        Point::new(
            first.x() + (along * angle.cos()).round() as i64,
            first.y() + (along * angle.sin()).round() as i64,
        )
    }
}

pub(crate) struct EdgeCache {
    contour: Contour,
    holes: Vec<Contour>,
    accuracy: f64,
}

impl EdgeCache {
    pub(crate) fn new(shape: &ExPolygon, accuracy: f64) -> Self {
        Self {
            contour: Contour::new(shape.contour().points()),
            holes: shape
                .holes()
                .iter()
                .map(|hole| Contour::new(hole.points()))
                .collect(),
            accuracy,
        }
    }

    /// `stride`: `round(N / N^(accuracy^(1/3)))`.
    fn stride(&self, count: usize) -> usize {
        let n = count as f64;
        (n / n.powf(self.accuracy.powf(1.0 / 3.0))).round() as i64 as usize
    }

    /// `fetchCorners`: two zero starts (upstream pushes 0.0 twice), then every stride-th vertex.
    pub(crate) fn corners(&mut self) -> &[f64] {
        if self.contour.corners.is_empty() {
            let count = self.contour.distances.len();
            let stride = self.stride(count);
            let mut corners = vec![0.0, 0.0];
            let mut i = 0;
            while i < count.wrapping_sub(1) {
                corners.push(self.contour.distances[i] / self.contour.full_distance);
                i += stride;
            }
            self.contour.corners = corners;
        }
        &self.contour.corners
    }

    /// `fetchHoleCorners`: one zero start, then every stride-th vertex.
    pub(crate) fn hole_corners(&mut self, hole: usize) -> &[f64] {
        if self.holes[hole].corners.is_empty() {
            let count = self.holes[hole].distances.len();
            let stride = self.stride(count);
            let mut corners = vec![0.0];
            let mut i = 0;
            while i < count.wrapping_sub(1) {
                corners.push(self.holes[hole].distances[i] / self.holes[hole].full_distance);
                i += stride;
            }
            self.holes[hole].corners = corners;
        }
        &self.holes[hole].corners
    }

    pub(crate) fn coords(&self, distance: f64) -> Point {
        self.contour.coords(distance)
    }

    pub(crate) fn hole_coords(&self, hole: usize, distance: f64) -> Point {
        self.holes[hole].coords(distance)
    }

    pub(crate) fn hole_count(&self) -> usize {
        self.holes.len()
    }
}
