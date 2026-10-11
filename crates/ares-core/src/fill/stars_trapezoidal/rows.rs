use super::SurfaceGeometry;
use crate::{
    fill::{checked_rotate::rotate_points, multiline::MultilineFillParams},
    geometry::{ClipperError, CoordinateScale, Point, Polyline},
};

pub(super) fn generate(
    params: MultilineFillParams,
    geometry: SurfaceGeometry,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let SurfaceGeometry {
        layer_id,
        thickness_layers,
        mut bounds,
    } = geometry;
    // FillRectilinear.cpp:3060/3073: truncate scaled spacing BEFORE multiplying.
    let d1 = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let period = ((2.0 * d1 as f64 / f64::from(params.density)) * 3.0_f64.sqrt()) as i64;
    let hex_height = (0.5 * 3.0_f64.sqrt() * period as f64) as i64;
    let tri_height = hex_height / 2;
    let d1_half = d1 / 2;
    let chamfer_height = (3.0_f64.sqrt() * d1_half as f64) as i64;
    let d1_half_base = (d1_half as f64 / 3.0_f64.sqrt()) as i64;
    let half_period = period / 2;
    let quarter_period = period / 4;
    let center = bounds.center();
    bounds.merge_point(Point::new(
        center.x().div_euclid(period) * period,
        center.y().div_euclid(tri_height) * tri_height,
    ));
    let half_w = (bounds.max().x() - bounds.min().x()) / 2;
    let half_h = (bounds.max().y() - bounds.min().y()) / 2;
    let periods_x = (half_w as f64 / period as f64).ceil() as i64;
    let mut periods_y = (half_h as f64 / hex_height as f64).ceil() as i64;
    if periods_y % 2 != 0 {
        periods_y += 1;
    }
    let x_min = -periods_x * period - half_period;
    let x_max = periods_x * period - half_period;
    let y_min = -periods_y * hex_height - (2 * tri_height) / 3;
    let y_max = periods_y * hex_height - (2 * tri_height) / 3;
    let mut normal = Vec::with_capacity(((x_max - x_min) / period) as usize * 7);
    let mut x = x_min;
    while x < x_max {
        normal.extend([
            Point::new(x, hex_height),
            Point::new(x + quarter_period - d1, hex_height),
            Point::new(x + quarter_period + d1_half, hex_height - chamfer_height),
            Point::new(x + half_period - d1_half_base, tri_height + d1_half),
            Point::new(x + half_period + d1_half_base, tri_height + d1_half),
            Point::new(x + (period * 3) / 4 - d1_half, hex_height - chamfer_height),
            Point::new(x + (period * 3) / 4 + d1, hex_height),
        ]);
        x += period;
    }
    let mirrored = normal
        .iter()
        .map(|point| Point::new(point.x(), hex_height - point.y()))
        .collect::<Vec<_>>();
    let infill_layer = if thickness_layers > 0 {
        layer_id / usize::from(thickness_layers)
    } else {
        layer_id
    };
    let layer_mod = infill_layer % 3;
    let angle = layer_mod as f64 * 2.0 * std::f64::consts::PI / 3.0;
    let mut lines = Vec::with_capacity(((y_max - y_min) / hex_height + 3) as usize * 2);
    let mut y = y_min;
    let mut pair = 0;
    while y < y_max {
        let x_shift = if pair % 2 == 0 { 0 } else { half_period };
        for template in [&normal, &mirrored] {
            let mut points = template
                .iter()
                .map(|p| Point::new(p.x() + x_shift + half_period, p.y() + y + tri_height))
                .collect::<Vec<_>>();
            if layer_mod != 0 {
                points = rotate_points(points, angle)?;
            }
            lines.push(Polyline::new(points));
        }
        pair += 1;
        y += hex_height;
    }
    Ok(lines)
}
