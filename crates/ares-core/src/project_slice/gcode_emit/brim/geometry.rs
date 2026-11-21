use crate::{
    SliceError,
    geometry::{Coord, CoordinateScale},
};

pub(super) fn scaled_coord(scale: CoordinateScale, value: f64) -> Result<Coord, SliceError> {
    scale
        .checked_scale(value)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

pub(crate) fn scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, SliceError> {
    scaled_coord(scale, value).map(|value| value as f32)
}

pub(crate) fn scaled_f64(scale: CoordinateScale, value: f64) -> Result<f64, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f64)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

pub(crate) fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".to_owned())
}
