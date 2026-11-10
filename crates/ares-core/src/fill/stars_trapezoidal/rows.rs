use crate::geometry::{BoundingBox, Point, Polyline};

/// `FillRectilinear.cpp:3317-3377`: the origin-anchored Stars rows over `cover`.
pub(super) fn generate(cover: BoundingBox, period: i64, d1: i64) -> Vec<Polyline> {
    let hex_height = (0.5 * 3.0_f64.sqrt() * period as f64) as i64;
    let tri_height = hex_height / 2;
    let d1_half = d1 / 2;
    let chamfer_height = (3.0_f64.sqrt() * d1_half as f64) as i64;
    let d1_half_base = (d1_half as f64 / 3.0_f64.sqrt()) as i64;
    let half_period = period / 2;
    let quarter_period = period / 4;
    let row_y_offset = tri_height - (2 * tri_height) / 3;
    let first_tile = (cover.min().x() as f64 / period as f64).floor() as i64 - 1;
    let last_tile = (cover.max().x() as f64 / period as f64).ceil() as i64 + 1;
    let first_row =
        ((cover.min().y() - row_y_offset) as f64 / hex_height as f64).floor() as i64 - 1;
    let last_row = ((cover.max().y() - row_y_offset) as f64 / hex_height as f64).ceil() as i64 + 1;
    let normal = (first_tile..last_tile)
        .flat_map(|tile| {
            let x = tile * period - half_period;
            [
                Point::new(x, hex_height),
                Point::new(x + quarter_period - d1, hex_height),
                Point::new(x + quarter_period + d1_half, hex_height - chamfer_height),
                Point::new(x + half_period - d1_half_base, tri_height + d1_half),
                Point::new(x + half_period + d1_half_base, tri_height + d1_half),
                Point::new(x + (period * 3) / 4 - d1_half, hex_height - chamfer_height),
                Point::new(x + (period * 3) / 4 + d1, hex_height),
            ]
        })
        .collect::<Vec<_>>();
    let mirrored = normal
        .iter()
        .map(|point| Point::new(point.x(), hex_height - point.y()))
        .collect::<Vec<_>>();
    (first_row..=last_row)
        .flat_map(|row| {
            let y = row * hex_height + row_y_offset;
            let x_shift = if row % 2 == 0 { 0 } else { half_period } + half_period;
            [&normal, &mirrored].map(|template| {
                Polyline::new(
                    template
                        .iter()
                        .map(|point| Point::new(point.x() + x_shift, point.y() + y))
                        .collect(),
                )
            })
        })
        .collect()
}
