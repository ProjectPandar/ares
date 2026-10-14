//! Polyline3 / ExtrusionEntity.hpp carrier; consumes owned XY points without point clones.
use super::types::{FittedMove, Point3, Polyline3};
use crate::geometry::{Point, Polyline};
impl Point3 {
    pub(in crate::project_slice) const fn x(&self) -> i64 {
        self.x
    }
    pub(in crate::project_slice) const fn y(&self) -> i64 {
        self.y
    }
}
impl Polyline3 {
    pub(in crate::project_slice) fn from_xy(polyline: Polyline, fitting: Vec<FittedMove>) -> Self {
        let source = polyline.into_points();
        let mut points = Vec::with_capacity(source.capacity());
        points.extend(source.into_iter().map(|point| Point3 {
            x: point.x(),
            y: point.y(),
            z: 0,
        }));
        Self { points, fitting }
    }
    #[cfg(test)]
    pub(in crate::project_slice) fn is_valid(&self) -> bool {
        self.points.len() >= 2
    }
    pub(in crate::project_slice) fn points(&self) -> &[Point3] {
        &self.points
    }
    pub(in crate::project_slice) fn front(&self) -> Option<Point> {
        self.points.first().map(|p| Point::new(p.x, p.y))
    }
    pub(in crate::project_slice) fn back(&self) -> Option<Point> {
        self.points.last().map(|p| Point::new(p.x, p.y))
    }
    pub(in crate::project_slice) fn reverse(&mut self) {
        let last_index = self.points.len().saturating_sub(1);
        for fitted in &mut self.fitting {
            let start = fitted.start;
            fitted.start = last_index - fitted.end;
            fitted.end = last_index - start;
            if let Some(arc) = &mut fitted.arc {
                arc.clockwise = !arc.clockwise;
            }
        }
        self.fitting.reverse();
        self.points.reverse();
    }
}
