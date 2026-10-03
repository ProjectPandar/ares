//! `TreeSupport3D.cpp::polyline_sample_next_point_at_distance`.

use crate::geometry::{Coord, CoordinateScale, Point};

pub(super) fn sample(
    polyline: &[Point],
    start: Point,
    start_index: usize,
    distance: f64,
    scale: CoordinateScale,
) -> Option<(Point, usize)> {
    let distance_squared = distance * distance;
    let distance_squared_integer = distance_squared as i64;
    let epsilon = scale.scaled_delta(0.01);
    for index in start_index + 1..polyline.len() {
        let last = polyline[index];
        let dx = last.x() - start.x();
        let dy = last.y() - start.y();
        if dx * dx + dy * dy < distance_squared_integer {
            continue;
        }
        let first = polyline[index - 1];
        let vx = (last.x() - first.x()) as f64;
        let vy = (last.y() - first.y()) as f64;
        let length_squared = vx * vx + vy * vy;
        if length_squared < epsilon * epsilon {
            let center = Point::new((first.x() + last.x()) / 2, (first.y() + last.y()) / 2);
            if (super::distance_squared(start, center).sqrt() - distance).abs() < epsilon {
                return Some((center, index - 1));
            }
            continue;
        }
        let px = (start.x() - first.x()) as f64;
        let py = (start.y() - first.y()) as f64;
        let projection = (px * vx + py * vy) / length_squared;
        let foot_x = vx * projection;
        let foot_y = vy * projection;
        let from_line_x = px - foot_x;
        let from_line_y = py - foot_y;
        let from_line_squared = from_line_x * from_line_x + from_line_y * from_line_y;
        let intersection_squared = distance_squared - from_line_squared;
        if intersection_squared > -scale.scaled_delta(1e-4) {
            let intersection_squared = intersection_squared.max(0.0);
            let tail_x = vx - foot_x;
            let tail_y = vy - foot_y;
            if tail_x * tail_x + tail_y * tail_y >= intersection_squared {
                let along = (intersection_squared / length_squared).sqrt();
                // Native truncates the offset before adding the integer segment origin.
                let point = Point::new(
                    first.x() + (foot_x + vx * along) as Coord,
                    first.y() + (foot_y + vy * along) as Coord,
                );
                return Some((point, index - 1));
            }
        }
    }
    None
}
