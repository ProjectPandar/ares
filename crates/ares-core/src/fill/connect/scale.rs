use crate::geometry::{BoundingBox, ClipperError, Coord, CoordinateScale, Point};

pub(super) const EPSILON_MM: f64 = 1e-4;

const MIN_COORD: f64 = i64::MIN as f64;
const MAX_COORD_EXCLUSIVE: f64 = -MIN_COORD;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IntegerBounds {
    pub(super) min: Point,
    pub(super) max: Point,
}

pub(super) fn scaled_f64(value: f64, scale: CoordinateScale) -> f64 {
    value / scale.factor()
}

pub(super) fn scaled_f32(value: f32, scale: CoordinateScale) -> f64 {
    f64::from(value) / scale.factor()
}

pub(super) fn scaled_epsilon(scale: CoordinateScale) -> f64 {
    scaled_f64(EPSILON_MM, scale)
}

pub(super) fn coord_from_completed(value: f64) -> Result<Coord, ClipperError> {
    if value.is_finite() && (MIN_COORD..MAX_COORD_EXCLUSIVE).contains(&value) {
        Ok(value.trunc() as Coord)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}

pub(super) fn scaled_coord_f64(value: f64, scale: CoordinateScale) -> Result<Coord, ClipperError> {
    coord_from_completed(scaled_f64(value, scale))
}

pub(super) fn inflate_bbox_round_delta(
    bbox: BoundingBox,
    delta: f64,
) -> Result<IntegerBounds, ClipperError> {
    let delta = coord_from_completed(delta.round())?;
    let min = bbox.min();
    let max = bbox.max();
    Ok(IntegerBounds {
        min: Point::new(
            min.x()
                .checked_sub(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            min.y()
                .checked_sub(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        ),
        max: Point::new(
            max.x()
                .checked_add(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            max.y()
                .checked_add(delta)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        ),
    })
}
