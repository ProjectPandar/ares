//! `FillRectilinear.cpp:3048-3162,3344-3384`: Grid multiline mode0.
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
    // FillRectilinear.cpp:3065/3070: promote the float direction before pi/4.
    let angle = f64::from(params.angle) + std::f64::consts::FRAC_PI_4;
    let rotated = rotate_surface(surface, -angle)?;
    let lines = rows::generate(params, geometry, scale);
    finish(rotated, lines, params, angle, scale)
}
