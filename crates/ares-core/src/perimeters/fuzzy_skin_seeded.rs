use super::{
    fuzzy_skin::{FuzzySkinConfig, FuzzySkinNoiseType},
    fuzzy_skin_coherent_noise::coherent_value,
};
use crate::{
    Point2,
    geometry::{CoordinateScale, Point},
    slicer_context::Mt19937,
};

pub(super) fn fuzzify(
    (points, mut closed): (&[Point], bool),
    config: FuzzySkinConfig,
    print_z: f64,
    scale: CoordinateScale,
    rng: &mut Mt19937,
) -> Vec<Point> {
    let step = scale.checked_scale(config.point_distance_mm).unwrap() as f64;
    let thickness = scale.checked_scale(config.thickness_mm).unwrap() as f64;
    let minimum = step * 3.0 / 4.0;
    let range = step / 2.0;
    let mut leftover = rng.unit() * (minimum / 2.0);
    let mut previous = *points.last().unwrap();
    let mut output = Vec::with_capacity(points.len());
    for &current in points {
        if !closed {
            closed = true;
            previous = current;
            continue;
        }
        let dx = (current.x() - previous.x()) as f64;
        let dy = (current.y() - previous.y()) as f64;
        let length = (dx * dx + dy * dy).sqrt();
        let mut distance = leftover;
        while distance < length {
            let ratio = distance / length;
            let base = Point::new(
                previous.x() + (dx * ratio) as i64,
                previous.y() + (dy * ratio) as i64,
            );
            let noise = match config.noise_type {
                FuzzySkinNoiseType::Classic => rng.unit() * 2.0 - 1.0,
                kind @ (FuzzySkinNoiseType::Perlin
                | FuzzySkinNoiseType::Billow
                | FuzzySkinNoiseType::RidgedMulti
                | FuzzySkinNoiseType::Voronoi) => coherent_value(
                    kind,
                    Point2::new(scale.unscale(base.x()), scale.unscale(base.y())),
                    print_z,
                    config,
                ),
                FuzzySkinNoiseType::Ripple => {
                    unreachable!("ripple is materialized without random noise")
                }
            };
            let displacement = noise * thickness;
            output.push(Point::new(
                base.x() + ((-dy / length) * displacement) as i64,
                base.y() + ((dx / length) * displacement) as i64,
            ));
            distance += minimum + rng.unit() * range;
        }
        leftover = distance - length;
        previous = current;
    }
    while output.len() < 3 {
        let point_index = points.len() - 2;
        output.push(points[point_index]);
        if point_index == 0 {
            break;
        }
    }
    if output.len() >= 3 {
        output
    } else {
        points.to_vec()
    }
}
