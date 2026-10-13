use super::SurfaceGeometry;
use crate::{
    fill::multiline::MultilineFillParams,
    geometry::{CoordinateScale, Point, Polyline},
};

pub(super) fn generate(
    params: MultilineFillParams,
    geometry: SurfaceGeometry,
    scale: CoordinateScale,
) -> Vec<Polyline> {
    let SurfaceGeometry {
        layer_id,
        thickness_layers,
        mut bounds,
    } = geometry;
    // FillRectilinear.cpp:3060/3069/3099: cast before count, then double period/d2.
    let d1 = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let period = ((2.0 * d1 as f64 / f64::from(params.density)) * std::f64::consts::SQRT_2) as i64;
    let d2 = (0.5 * period as f64 - d1 as f64) as i64;
    // Point.hpp:709-720 / FillRectilinear.cpp:3102: floor-align bb.min, not center.
    bounds.merge_point(Point::new(
        bounds.min().x().div_euclid(period) * period,
        bounds.min().y().div_euclid(period) * period,
    ));
    let xmin = bounds.min().x();
    let xmax = bounds.max().x();
    let ymin = bounds.min().y();
    let ymax = bounds.max().y();
    let mut normal = Vec::with_capacity((((xmax - xmin) / period + 1) * 5) as usize);
    let mut x = xmin;
    while x < xmax {
        normal.extend([
            Point::new(x, d1 / 2),
            Point::new(x + d1 / 2, d1 / 2),
            Point::new(x + d1 / 2 + d2, d1 / 2 + d2),
            Point::new(x + d1 / 2 + d2 + d1, d1 / 2 + d2),
            Point::new(x + period - d1 / 2, d1 / 2),
        ]);
        x += period;
    }
    let flipped = normal
        .iter()
        .map(|p| Point::new(p.x(), period / 2 - p.y()))
        .collect::<Vec<_>>();
    let mut lines = Vec::with_capacity(((ymax - ymin) / (period / 2) + 1) as usize);
    let mut flip = false;
    let mut y = ymin;
    while y < ymax {
        let template = if flip { &flipped } else { &normal };
        lines.push(Polyline::new(
            template
                .iter()
                .map(|p| Point::new(p.x(), p.y() + y))
                .collect(),
        ));
        flip = !flip;
        y += period / 2;
    }
    let infill_layer = if thickness_layers > 0 {
        layer_id / usize::from(thickness_layers)
    } else {
        layer_id
    };
    if infill_layer % 2 == 1 {
        for line in &mut lines {
            let mut points = std::mem::replace(line, Polyline::new(Vec::new())).into_points();
            for point in &mut points {
                *point = Point::new(point.y(), point.x());
            }
            *line = Polyline::new(points);
        }
    }
    lines
}
