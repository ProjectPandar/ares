//! FillRectilinear.cpp:3100+ common frame and mode 3 cubic rows.
mod envelope;
mod rows;

use super::{
    checked_rotate::rotate_points,
    multiline::MultilineFillParams,
    trapezoidal_tail::{finish, rotate_surface},
};
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polyline};

pub(crate) struct SurfaceGeometry {
    pub(crate) layer_id: usize,
    pub(crate) thickness_layers: u16,
    pub(crate) print_z: f64,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MultilineFillParams,
    geometry: SurfaceGeometry,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let d1 = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let period = ((2.0 * d1 as f64 / f64::from(params.density)) * 3.0_f64.sqrt()) as i64;
    let base_angle = f64::from(params.angle) + std::f64::consts::FRAC_PI_2;
    let rotated = rotate_surface(surface, -base_angle)?;
    let infill_layer = if geometry.thickness_layers > 0 {
        geometry.layer_id / usize::from(geometry.thickness_layers)
    } else {
        geometry.layer_id
    };
    let layer_mod = infill_layer % 3;
    let angle = layer_mod as f64 * 2.0 * std::f64::consts::PI / 3.0;
    let mut local = rotated
        .contour()
        .points()
        .iter()
        .map(|point| {
            Point::new(
                point.x() - params.reference.x(),
                point.y() - params.reference.y(),
            )
        })
        .collect::<Vec<_>>();
    if layer_mod != 0 {
        local = rotate_points(local, -angle)?;
    }
    let mut cover = BoundingBox::from_points(local.into_iter())
        .expect("a fill surface retains its nonempty contour");
    cover.offset(period);
    let shift = (0.5_f64.sqrt() * geometry.print_z) / scale.factor();
    let mut polylines = rows::generate(cover, period, d1, shift)?;
    for line in &mut polylines {
        let mut points = std::mem::replace(line, Polyline::new(Vec::new())).into_points();
        if layer_mod != 0 {
            points = rotate_points(points, angle)?;
        }
        for point in &mut points {
            *point = Point::new(
                point.x() + params.reference.x(),
                point.y() + params.reference.y(),
            );
        }
        *line = Polyline::new(points);
    }
    finish(
        rotated,
        polylines,
        params,
        (base_angle, [angle.cos(), angle.sin()]),
        scale,
    )
}
