//! Endpoint preparation/merging (`FillAdaptive.cpp:815-880,1074-1108`).

use super::{
    geometry::{distance_point_to_segment_squared, distance_squared},
    pair::resolve_merged_index,
};
use crate::geometry::{Line, Point};

pub(super) fn collect(working: &mut [Vec<Point>]) -> Vec<(usize, usize)> {
    let mut indexed = Vec::with_capacity(working.len());
    let mut touching = Vec::new();
    for current in 0..working.len() {
        if current > 0 {
            let front = working[current][0];
            let back = *working[current].last().unwrap();
            let collinear_front =
                collinear_segment((front, back), current, working, &mut indexed, &mut touching);
            let collinear_back =
                collinear_segment((back, front), current, working, &mut indexed, &mut touching);
            if let Some((other, front)) = collinear_front {
                let point = far_endpoint(&working[other], front);
                working[current][0] = point;
                working[other].clear();
            }
            if let Some((other, front)) = collinear_back {
                let point = far_endpoint(&working[other], front);
                *working[current].last_mut().unwrap() = point;
                working[other].clear();
            }
        }
        indexed.push(Some(Line::new(
            index_point(working[current][0]),
            index_point(*working[current].last().unwrap()),
        )));
    }
    touching.sort_unstable();
    touching.dedup();
    touching
}

fn far_endpoint(points: &[Point], near_front: bool) -> Point {
    if near_front {
        *points.last().unwrap()
    } else {
        points[0]
    }
}

fn collinear_segment(
    (point, other_point): (Point, Point),
    current: usize,
    working: &[Vec<Point>],
    indexed: &mut [Option<Line>],
    touching: &mut Vec<(usize, usize)>,
) -> Option<(usize, bool)> {
    let query = index_point(point);
    let mut closest = None;
    let mut best_distance = f64::INFINITY;
    for (index, line) in indexed.iter().enumerate() {
        let Some(line) = line else { continue };
        let distance = distance_point_to_segment_squared(query, line);
        if distance < best_distance {
            closest = Some(index);
            best_distance = distance;
        }
    }
    let other = closest?;
    let front = working[other][0];
    let back = *working[other].last().unwrap();
    let distance_front = distance_squared(front, point);
    let distance_back = distance_squared(back, point);
    let minimum = distance_front.min(distance_back);
    let r2_close = 1200.0 * 1200.0;
    if minimum < r2_close {
        let minimum_other =
            distance_squared(front, other_point).min(distance_squared(back, other_point));
        if minimum_other > minimum {
            let v1 = (
                (other_point.x() - point.x()) as f64,
                (other_point.y() - point.y()) as f64,
            );
            let v2 = ((back.x() - front.x()) as f64, (back.y() - front.y()) as f64);
            let n1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
            let n2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
            let d = (v1.0 / n1) * (v2.0 / n2) + (v1.1 / n1) * (v2.1 / n2);
            if d.abs() > f64::from(0.99_f32) {
                // Native removes the segment before querying the other endpoint.
                indexed[other] = None;
                return Some((other, minimum == distance_front));
            }
            touching.push((current.min(other), current.max(other)));
        }
    }
    None
}

// `mk_rtree_point` (`FillAdaptive.cpp:654-663`) stores float coordinates.
fn index_point(point: Point) -> Point {
    Point::new(point.x() as f32 as i64, point.y() as f32 as i64)
}

pub(super) fn merge(
    working: &mut [Vec<Point>],
    touching: &[(usize, usize)],
    merged_with: &mut [usize],
) {
    for &(first_index, second_index) in touching.iter().rev() {
        let second_index = resolve_merged_index(merged_with, second_index);
        if first_index == second_index
            || working[first_index].is_empty()
            || working[second_index].is_empty()
        {
            continue;
        }
        let (head, tail) = working.split_at_mut(second_index);
        let first = &mut head[first_index];
        let second = &mut tail[0];
        let d11 = distance_squared(first[0], second[0]);
        let d12 = distance_squared(first[0], *second.last().unwrap());
        let d21 = distance_squared(*first.last().unwrap(), second[0]);
        let d22 = distance_squared(*first.last().unwrap(), *second.last().unwrap());
        let d1min = d11.min(d12);
        let d2min = d21.min(d22);
        if d1min < d2min {
            first.reverse();
            if d12 == d1min {
                second.reverse();
            }
        } else if d22 == d2min {
            second.reverse();
        }
        let last = first.len() - 1;
        // Point's Eigen coord_t expression adds integers, then divides toward zero.
        first[last] = Point::new(
            (first[last].x() + second[0].x()) / 2,
            (first[last].y() + second[0].y()) / 2,
        );
        first.extend_from_slice(&second[1..]);
        second.clear();
        merged_with[second_index] = first_index;
    }
}
