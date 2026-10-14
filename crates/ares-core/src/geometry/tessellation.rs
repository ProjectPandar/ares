//! `libslic3r/Tesselate.cpp:23–56/100–170/192–195` default GLU boundary.
mod kernel;
#[cfg(test)]
mod tests;

use super::{CoordinateScale, ExPolygon};

pub(crate) fn triangulate_expolygon_3d(
    expolygon: &ExPolygon,
    z: f64,
    flip: bool,
    scale: CoordinateScale,
) -> Vec<[f64; 3]> {
    let contours: Vec<_> = std::iter::once(expolygon.contour())
        .chain(expolygon.holes())
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .map(|point| [scale.unscale(point.x()), scale.unscale(point.y()), 0.0])
                .collect()
        })
        .collect();
    kernel::tessellate(&contours, z, flip)
}
