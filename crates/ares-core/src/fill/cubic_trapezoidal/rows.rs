use super::envelope::upper_level;
use crate::{
    fill::checked_rotate::checked_point,
    geometry::{BoundingBox, ClipperError, Polyline},
};

// FillRectilinear.cpp:3379-3411: paired, staggered cubic levels.
pub(super) fn generate(
    cover: BoundingBox,
    period: i64,
    d1: i64,
    shift: f64,
) -> Result<Vec<Polyline>, ClipperError> {
    let pos_mod = |a: f64, m: f64| {
        let r = a % m;
        if r < 0.0 { r + m } else { r }
    };
    let h = 0.5 * 3.0_f64.sqrt() * period as f64;
    let tau = pos_mod(-3.0 * shift, h);
    let y0 = pos_mod(-2.0 * shift, 2.0 * h);
    let mut levels = [
        upper_level(h - tau, h, period as f64, d1 as f64),
        upper_level(tau, h, period as f64, d1 as f64),
    ];
    for point in &mut levels[0] {
        point[1] = h - point[1];
    }
    let n_min = ((cover.min().y() as f64 - y0) / h).floor() as i64 - 1;
    let n_max = ((cover.max().y() as f64 - y0) / h).ceil() as i64 + 1;
    let mut polylines = Vec::with_capacity((n_max - n_min + 1) as usize * levels.len());
    for n in n_min..=n_max {
        let x_off = if n & 1 != 0 { 0.5 * period as f64 } else { 0.0 };
        let base = y0 + n as f64 * h - tau;
        let j_min = ((cover.min().x() as f64 - x_off) / period as f64).floor() as i64 - 1;
        let j_max = ((cover.max().x() as f64 - x_off) / period as f64).ceil() as i64;
        for level in &levels {
            let mut points = Vec::with_capacity((j_max - j_min + 1) as usize * level.len());
            let tiled = (j_min..=j_max).flat_map(|j| {
                let first = usize::from(j != j_min);
                level[first..].iter().map(move |point| (j, point))
            });
            for (j, point) in tiled {
                points.push(checked_point(
                    x_off + (j * period) as f64 + point[0],
                    base + point[1],
                )?);
            }
            polylines.push(Polyline::new(points));
        }
    }
    Ok(polylines)
}
