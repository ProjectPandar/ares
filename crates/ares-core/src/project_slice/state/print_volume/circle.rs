//! BuildVolume.cpp:38-63 / Geometry/Circle.cpp:111-139.
use crate::geometry::enclosing_circle::circle_center;
use crate::{Point2d, slicer_context::Mt19937};

use super::{Circle, SCENE_EPSILON};

pub(super) fn fit_circle(points: &[Point2d]) -> Option<Circle> {
    if points.len() <= 3 {
        return None;
    }
    let mut rng = Mt19937::default();
    let mut best = Circle {
        center: [0.0; 2],
        radius: -1.0,
    };
    let mut minimum_error = f64::MAX;
    for _ in 0..20 {
        let [a, b, c] = sample_three(points.len(), &mut rng);
        let point = |index: usize| [points[index].x, points[index].y];
        let center = circle_center(point(a), point(b), point(c), SCENE_EPSILON);
        let radius =
            points.iter().fold(0.0, |sum, p| distance(p, center) + sum) / points.len() as f64;
        let error = points.iter().fold(0.0_f64, |error, p| {
            error.max((distance(p, center) - radius).abs())
        });
        if error < minimum_error {
            minimum_error = error;
            best = Circle { center, radius };
        }
    }
    let mut previous = &points[points.len() - 1];
    for point in points {
        let midpoint = Point2d {
            x: 0.5 * (previous.x + point.x),
            y: 0.5 * (previous.y + point.y),
        };
        if (distance(point, best.center) - best.radius).abs() > 0.005
            || best.radius - distance(&midpoint, best.center) > 3.0
        {
            return None;
        }
        previous = point;
    }
    let scale = crate::geometry::CoordinateScale::Normal;
    Some(Circle {
        center: best.center.map(|c| scale.scaled_delta(c)),
        radius: scale.scaled_delta(best.radius),
    })
}

fn distance(p: &Point2d, center: [f64; 2]) -> f64 {
    let x = p.x - center[0];
    let y = p.y - center[1];
    (x * x + y * y).sqrt()
}

/// libstdc++15 stl_algo.h:5834-5900 forward-input selection sampling.
/// std::sample preserves input order and batches two draws when the RNG range permits.
fn sample_three(size: usize, rng: &mut Mt19937) -> [usize; 3] {
    let mut result = [0; 3];
    let mut selected = 0;
    let mut position = 0;
    let mut remaining = size;
    let paired = u64::from(u32::MAX) / size as u64 >= size as u64;
    while selected != 3 {
        let (draws, count) = if paired && remaining >= 2 {
            let combined = rng.integer(0, (remaining * (remaining - 1) - 1) as i32) as u32 as usize;
            ([combined / (remaining - 1), combined % (remaining - 1)], 2)
        } else {
            (
                [rng.integer(0, (remaining - 1) as i32) as u32 as usize, 0],
                1,
            )
        };
        for draw in draws.into_iter().take(count) {
            remaining -= 1;
            if draw < 3 - selected {
                result[selected] = position;
                selected += 1;
            }
            position += 1;
            if selected == 3 {
                break;
            }
        }
    }
    result
}
