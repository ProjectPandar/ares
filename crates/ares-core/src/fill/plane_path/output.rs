use super::bounds::Bounds;
use crate::geometry::{ClipperError, Point};

const LEFT: i32 = 1;
const RIGHT: i32 = 2;
const TOP: i32 = 4;
const BOTTOM: i32 = 8;
const MIN_COORDINATE: f64 = i64::MIN as f64;
const MAX_COORDINATE_EXCLUSIVE: f64 = -MIN_COORDINATE;

/// `InfillPolylineOutput` / `InfillPolylineClipper` from
/// OrcaSlicer 2.4.2 `FillPlanePath.hpp` and `FillPlanePath.cpp:9-65`.
pub(super) struct InfillPolylineOutput {
    scale_out: f64,
    clip: Option<Bounds>,
    points: Vec<Point>,
    sides_previous: i32,
    sides_current: i32,
}

impl InfillPolylineOutput {
    pub(super) fn plain(scale_out: f64) -> Self {
        Self::new(scale_out, None)
    }

    pub(super) fn clipped(bounds: Bounds, scale_out: f64) -> Self {
        Self::new(scale_out, Some(bounds))
    }

    fn new(scale_out: f64, clip: Option<Bounds>) -> Self {
        Self {
            scale_out,
            clip,
            points: Vec::new(),
            sides_previous: 0,
            sides_current: 0,
        }
    }

    pub(super) fn reserve(&mut self, count: usize) {
        self.points.reserve(count);
    }

    pub(super) fn add_point(&mut self, x: f64, y: f64) -> Result<(), ClipperError> {
        let point = Point::new(self.scaled(x)?, self.scaled(y)?);
        let Some(bounds) = self.clip else {
            self.points.push(point);
            return Ok(());
        };

        if self.points.len() < 2 {
            if self.points.is_empty() {
                self.sides_previous = sides(bounds, point);
            } else {
                self.sides_current = sides(bounds, point);
            }
            self.points.push(point);
            return Ok(());
        }

        let sides_next = sides(bounds, point);
        if self.sides_current == 0 || self.sides_previous & self.sides_current & sides_next == 0 {
            self.sides_previous = self.sides_current;
        } else {
            self.points.pop();
        }
        self.points.push(point);
        self.sides_current = sides_next;
        Ok(())
    }

    pub(super) fn result(self) -> Vec<Point> {
        self.points
    }

    fn scaled(&self, value: f64) -> Result<i64, ClipperError> {
        let scaled = (value * self.scale_out + 0.5).floor();
        if scaled.is_finite() && (MIN_COORDINATE..MAX_COORDINATE_EXCLUSIVE).contains(&scaled) {
            Ok(scaled as i64)
        } else {
            Err(ClipperError::CoordinateOutOfRange)
        }
    }
}

fn sides(bounds: Bounds, point: Point) -> i32 {
    (point.x() < bounds.minimum().x()) as i32 * LEFT
        + (point.x() > bounds.maximum().x()) as i32 * RIGHT
        + (point.y() < bounds.minimum().y()) as i32 * BOTTOM
        + (point.y() > bounds.maximum().y()) as i32 * TOP
}
