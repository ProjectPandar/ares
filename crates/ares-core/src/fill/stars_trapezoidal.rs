//! `FillRectilinear.cpp:3048-3085/3260-3384`: Stars multiline mode2.
mod rows;

use super::{
    multiline::MultilineFillParams,
    trapezoidal_tail::{finish, rotate_surface},
};
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, Polyline};

pub(crate) struct SurfaceGeometry {
    pub(crate) layer_id: usize,
    pub(crate) thickness_layers: u16,
    pub(crate) bounds: BoundingBox,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MultilineFillParams,
    geometry: SurfaceGeometry,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // _infill_direction returns float; the caller adds M_PI_2 in double.
    let angle = f64::from(params.angle) + std::f64::consts::FRAC_PI_2;
    let rotated = rotate_surface(surface, -angle)?;
    let lines = rows::generate(params, geometry, scale)?;
    finish(rotated, lines, params, angle, scale)
}
