use crate::{SliceError, geometry::CoordinateScale};

pub(crate) fn scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f32)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
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
