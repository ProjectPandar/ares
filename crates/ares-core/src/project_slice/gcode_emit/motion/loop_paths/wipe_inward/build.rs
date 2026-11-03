//! `WipePathHelpers.cpp:97-187`: native offset segments, miter and bevel joins.

use super::{
    Params,
    math::{EPSILON, MITER_LIMIT, REVERSAL_DOT, Vector},
};
use crate::geometry::{Line, Point};

pub(super) fn polyline(original: &[Point], params: Params) -> Option<(Vec<Point>, usize)> {
    if original.len() < 2 {
        return None;
    }
    let mut source = original.to_vec();
    source.dedup();
    if source.len() < 2 {
        return None;
    }
    if source.windows(3).any(|points| {
        let previous = Vector::delta(points[1], points[0]);
        let next = Vector::delta(points[2], points[1]);
        previous.dot(next) / (previous.norm() * next.norm()) < REVERSAL_DOT
    }) {
        return None;
    }
    let mut result = Vec::with_capacity(source.len());
    let mut previous_line = segment(source[0], source[1], params);
    result.push(previous_line.a);
    let mut first_join = 0;
    for i in 1..source.len() - 1 {
        let next_line = segment(source[i], source[i + 1], params);
        if let Some(point) = miter(&source[i - 1..=i + 1], params) {
            result.push(point);
        } else {
            result.push(previous_line.b);
            if next_line.a != *result.last().unwrap() {
                result.push(next_line.a);
            }
        }
        if i == 1 {
            first_join = result.len() - 1;
        }
        previous_line = next_line;
    }
    result.push(previous_line.b);
    Some((result, first_join))
}

fn segment(a: Point, b: Point, params: Params) -> Line {
    let vector = Vector::delta(b, a);
    let distance = vector.norm();
    let perpendicular = if distance > params.epsilon {
        vector.left() * (f64::from(params.direction) * params.offset / distance)
    } else {
        Vector::default()
    };
    Line::new(
        (Vector::from_point(a) + perpendicular).point(),
        (Vector::from_point(b) + perpendicular).point(),
    )
}

fn miter(points: &[Point], params: Params) -> Option<Point> {
    let previous = Vector::delta(points[1], points[0]).normalized();
    let next = Vector::delta(points[2], points[1]).normalized();
    let denominator = 1.0 + previous.dot(next);
    if denominator <= EPSILON {
        return None;
    }
    let miter = (previous.left() + next.left())
        * (f64::from(params.direction) * params.offset / denominator);
    if miter.norm() > MITER_LIMIT * params.offset {
        return None;
    }
    Some((Vector::from_point(points[1]) + miter).point())
}
