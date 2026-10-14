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
    // FillRectilinear.cpp:3060/3073/3174-3175: preserve casts and double products.
    let d1 = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let period = ((2.0 * d1 as f64 / f64::from(params.density)) * 3.0_f64.sqrt()) as i64;
    let d2 = (2.0 / 3.0_f64.sqrt() * d1 as f64) as i64;
    let h = (0.5 * 3.0_f64.sqrt() * period as f64) as i64;
    let center = bounds.center();
    bounds.merge_point(Point::new(
        center.x().div_euclid(period) * period,
        center.y().div_euclid(h) * h,
    ));
    let half_w = (bounds.max().x() - bounds.min().x()) / 2;
    let half_h = (bounds.max().y() - bounds.min().y()) / 2;
    let periods_x = (half_w as f64 / period as f64).ceil() as i64;
    let mut periods_y = (half_h as f64 / h as f64).ceil() as i64;
    if periods_y % 2 != 0 {
        periods_y += 1;
    }
    let x_min = -periods_x * period;
    let x_max = periods_x * period;
    let y_min = -periods_y * h;
    let y_max = periods_y * h;
    // :3212-3227: five points per cell, with source integer half divisions.
    let mut normal = Vec::with_capacity(((x_max - x_min) / period + 1) as usize * 5);
    let mut x = x_min;
    while x < x_max {
        normal.extend([
            Point::new(x + d2 / 2, d1),
            Point::new(x + period / 2 - d2 / 2, h - d1),
            Point::new(x + period / 2 + d2 / 2, h - d1),
            Point::new(x + period - d2 / 2, d1),
            Point::new(x + period, d1),
        ]);
        x += period;
    }
    let mirrored = normal
        .iter()
        .map(|p| Point::new(p.x(), h - p.y()))
        .collect::<Vec<_>>();
    let infill_layer = if thickness_layers > 0 {
        layer_id / usize::from(thickness_layers)
    } else {
        layer_id
    };
    let layer_mod = infill_layer % 3;
    let angle = layer_mod as f64 * 2.0 * std::f64::consts::PI / 3.0;
    let rows = (y_max - y_min) / h + 2;
    let mut lines = Vec::with_capacity(((rows + 1) * 2) as usize);
    let mut shift = false;
    let mut y = y_min;
    while y < y_max {
        // :3233-3248: baseline precedes its normal or vertically mirrored row.
        let baseline = vec![Point::new(x_min, y), Point::new(x_max, y)];
        let template = if shift { &mirrored } else { &normal };
        let trapezoid = template
            .iter()
            .map(|p| Point::new(p.x(), p.y() + y))
            .collect::<Vec<_>>();
        lines.push(Polyline::new(baseline));
        if !trapezoid.is_empty() {
            lines.push(Polyline::new(trapezoid));
        }
        shift = !shift;
        y += h;
    }
    // :3252-3255 rotates about the origin, not the unused rotation_center.
    if layer_mod != 0 {
        for line in &mut lines {
            let points = std::mem::replace(line, Polyline::new(Vec::new())).into_points();
            *line = Polyline::new(rotate_points(points, angle)?);
        }
    }
    Ok(lines)
}
