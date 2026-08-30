use crate::{
    fill::checked_rotate::rotate_point,
    geometry::{BoundingBox, ClipperError, ExPolygon, Point},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Bounds {
    minimum: Point,
    maximum: Point,
}

impl Bounds {
    pub(super) const fn new(minimum: Point, maximum: Point) -> Self {
        Self { minimum, maximum }
    }

    pub(super) fn from_expolygon(expolygon: &ExPolygon) -> Self {
        Self::from_bounding_box(
            BoundingBox::from_expolygon(expolygon)
                .expect("a plane-path fill component has a nonempty contour"),
        )
    }

    pub(super) const fn from_bounding_box(bounds: BoundingBox) -> Self {
        Self::new(bounds.min(), bounds.max())
    }

    pub(super) const fn minimum(self) -> Point {
        self.minimum
    }

    pub(super) const fn maximum(self) -> Point {
        self.maximum
    }

    pub(super) fn center(self) -> Point {
        Point::new(
            midpoint(self.minimum.x(), self.maximum.x()),
            midpoint(self.minimum.y(), self.maximum.y()),
        )
    }

    pub(super) fn offset(&mut self, delta: i64) -> Result<(), ClipperError> {
        self.minimum = Point::new(
            self.minimum
                .x()
                .checked_sub(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            self.minimum
                .y()
                .checked_sub(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        );
        self.maximum = Point::new(
            self.maximum
                .x()
                .checked_add(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            self.maximum
                .y()
                .checked_add(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        );
        Ok(())
    }

    pub(super) fn translate(&mut self, delta: Point) -> Result<(), ClipperError> {
        self.minimum = add(self.minimum, delta)?;
        self.maximum = add(self.maximum, delta)?;
        Ok(())
    }

    pub(super) fn rotated(self, angle: f64) -> Result<Self, ClipperError> {
        let (cosine, sine) = (angle.cos(), angle.sin());
        let corners = [
            self.minimum,
            self.maximum,
            Point::new(self.minimum.x(), self.maximum.y()),
            Point::new(self.maximum.x(), self.minimum.y()),
        ];
        let mut rotated = corners
            .into_iter()
            .map(|point| rotate_point(point, cosine, sine));
        let first = rotated.next().expect("a bounding box has four corners")?;
        let mut bounds = Self::new(first, first);
        for point in rotated {
            let point = point?;
            bounds = Self::new(
                Point::new(
                    bounds.minimum.x().min(point.x()),
                    bounds.minimum.y().min(point.y()),
                ),
                Point::new(
                    bounds.maximum.x().max(point.x()),
                    bounds.maximum.y().max(point.y()),
                ),
            );
        }
        Ok(bounds)
    }
}

fn add(point: Point, delta: Point) -> Result<Point, ClipperError> {
    Ok(Point::new(
        point
            .x()
            .checked_add(delta.x())
            .ok_or(ClipperError::CoordinateOutOfRange)?,
        point
            .y()
            .checked_add(delta.y())
            .ok_or(ClipperError::CoordinateOutOfRange)?,
    ))
}

fn midpoint(first: i64, second: i64) -> i64 {
    ((i128::from(first) + i128::from(second)) / 2) as i64
}
