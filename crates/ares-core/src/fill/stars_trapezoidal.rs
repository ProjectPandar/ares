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
    let infill_layer = if geometry.thickness_layers > 0 {
        geometry.layer_id / usize::from(geometry.thickness_layers)
    } else {
        geometry.layer_id
    };
    let row_angle = (infill_layer % 3) as f64 * 2.0 * std::f64::consts::PI / 3.0;
    let row_direction = [row_angle.cos(), row_angle.sin()];
    let lines = rows::generate(params, geometry, scale)?;
    finish(rotated, lines, params, (angle, row_direction), scale)
}
