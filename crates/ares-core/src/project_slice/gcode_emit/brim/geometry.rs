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

pub(super) fn even_width_mm(width_mm: f64, spacing_mm: f64) -> f64 {
    (width_mm / spacing_mm / 2.0).floor() * spacing_mm * 2.0
}

pub(super) fn outer_width(width: f32, spacing: f32) -> f32 {
    // Brim.cpp:576: float divisions followed by double floor and products.
    (f64::from(width / spacing / 2.0).floor() * f64::from(spacing) * 2.0) as f32
}

pub(crate) fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".to_owned())
}
