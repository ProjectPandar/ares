//! `FillRectilinear.cpp:3048-3085,3160-3257,3344-3384`: Triangles mode1.
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
    // :3073-3074: promote _infill_direction's float before adding double pi/2.
    let angle = f64::from(params.angle) + std::f64::consts::FRAC_PI_2;
    let rotated = rotate_surface(surface, -angle)?;
    let lines = rows::generate(params, geometry, scale)?;
    finish(rotated, lines, params, angle, scale)
}
