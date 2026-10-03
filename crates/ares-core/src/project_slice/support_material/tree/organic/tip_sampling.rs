//! `TreeSupport3D.cpp::ensure_maximum_distance_polyline`.

mod next_point;

use crate::geometry::{Coord, CoordinateScale, Point, Polyline};

pub(super) fn ensure_distance(
    input: Vec<Polyline>,
    distance: f64,
    minimum_points: usize,
    scale: CoordinateScale,
) -> Vec<Polyline> {
    let mut result = Vec::with_capacity(input.len());
    let minimum_distance = scale.scaled_delta(0.1);
    for mut part in input {
        if part.points().is_empty() {
            continue;
        }
        let length = part
            .points()
            .windows(2)
            .map(|points| distance_squared(points[0], points[1]).sqrt())
            .sum::<f64>();
        if length < 2.0 * distance && minimum_points <= 1 {
            part.clip_end(length / 2.0);
            result.push(Polyline::new(vec![
                part.back().expect("half-length clipping retains a point"),
            ]));
            continue;
        }
        let mut line = Vec::new();
        let mut current_distance = distance.max(minimum_distance);
        let end_index = arrange_endpoints(part.points_mut());
        while line.len() < minimum_points && current_distance >= minimum_distance {
            sample_line(
                part.points(),
                &mut line,
                (end_index, minimum_points > 1),
                current_distance,
                scale,
            );
            current_distance *= 0.9;
        }
        result.push(Polyline::new(line));
    }
    result
}

fn arrange_endpoints(points: &mut [Point]) -> usize {
    let mut end = points.len() - 1;
    if points[0] != points[end] {
        return end;
    }
    let mut start = 0;
    let mut maximum_squared: Coord = 0;
    for index in 0..points.len() - 1 {
        for other in 0..points.len() - 1 {
            let squared = distance_squared(points[index], points[other]);
            if squared > maximum_squared as f64 {
                start = index;
                end = other;
                maximum_squared = squared as Coord;
            }
        }
    }
    let closing = points.len() - 1;
    points[..closing].rotate_left(start);
    points[closing] = points[0];
    (points.len() + end - start - 1) % closing
}

fn sample_line(
    points: &[Point],
    line: &mut Vec<Point>,
    endpoint: (usize, bool),
    current_distance: f64,
    scale: CoordinateScale,
) {
    let (end_index, force_end) = endpoint;
    let minimum_distance = scale.scaled_delta(0.1);
    line.clear();
    let mut current = points[0];
    line.push(current);
    if force_end || distance_squared(current, points[end_index]).sqrt() > current_distance {
        line.push(points[end_index]);
    }
    let mut index = 0;
    let mut next_distance = current_distance;
    while let Some((next, next_index)) =
        next_point::sample(points, current, index, next_distance, scale)
    {
        let minimum_existing = line.iter().fold(f64::MAX, |minimum, &point| {
            minimum.min(distance_squared(point, next).sqrt())
        });
        if minimum_existing >= current_distance {
            line.push(next);
            current = next;
            index = next_index;
            next_distance = current_distance;
        } else if current == next {
            if next_distance > 2.0 * current_distance {
                break;
            }
            next_distance += current_distance;
        } else {
            next_distance = (current_distance - minimum_existing).max(minimum_distance);
            current = next;
            index = next_index;
        }
    }
}

fn distance_squared(first: Point, second: Point) -> f64 {
    let dx = (first.x() - second.x()) as f64;
    let dy = (first.y() - second.y()) as f64;
    dx * dx + dy * dy
}
