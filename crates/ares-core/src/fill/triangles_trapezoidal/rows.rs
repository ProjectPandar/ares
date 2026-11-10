use crate::geometry::{BoundingBox, Point, Polyline};

/// `FillRectilinear.cpp:3237-3316`: the origin-anchored Triangles rows over `cover`.
pub(super) fn generate(cover: BoundingBox, period: i64, d1: i64) -> Vec<Polyline> {
    let d2 = (2.0 / 3.0_f64.sqrt() * d1 as f64) as i64;
    let h = (0.5 * 3.0_f64.sqrt() * period as f64) as i64;
    let x_min = ((cover.min().x() as f64 / period as f64).floor() as i64 - 1) * period;
    let x_max = ((cover.max().x() as f64 / period as f64).ceil() as i64 + 1) * period;
    let first_row = (cover.min().y() as f64 / h as f64).floor() as i64 - 1;
    let last_row = (cover.max().y() as f64 / h as f64).ceil() as i64 + 1;
    let normal = (0..)
        .map(|tile| x_min + tile * period)
        .take_while(|x| *x < x_max)
        .flat_map(|x| {
            [
                Point::new(x + d2 / 2, d1),
                Point::new(x + period / 2 - d2 / 2, h - d1),
                Point::new(x + period / 2 + d2 / 2, h - d1),
                Point::new(x + period - d2 / 2, d1),
                Point::new(x + period, d1),
            ]
        })
        .collect::<Vec<_>>();
    let mirrored = normal
        .iter()
        .map(|p| Point::new(p.x(), h - p.y()))
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    for row in first_row..=last_row {
        let y = row * h;
        lines.push(Polyline::new(vec![
            Point::new(x_min, y),
            Point::new(x_max, y),
        ]));
        let template = if row % 2 != 0 { &mirrored } else { &normal };
        if !template.is_empty() {
            lines.push(Polyline::new(
                template
                    .iter()
                    .map(|p| Point::new(p.x(), p.y() + y))
                    .collect(),
            ));
        }
    }
    lines
}
