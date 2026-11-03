//! `WipePathHelpers.cpp:195-402`: atomic stored offset/translated paths.

use super::{
    Params, build,
    math::{Vector, clip_end, length},
};
use crate::geometry::Point;

pub(super) fn backtracks(path: &[Point], start: Point, epsilon: f64) -> bool {
    if path.len() < 3 {
        return false;
    }
    let connector = Vector::delta(path[1], start);
    let outgoing = Vector::delta(path[2], path[1]);
    connector.dot(outgoing) < -4.0 * epsilon * outgoing.norm()
}

pub(super) fn store(mut actual: Vec<Point>, params: Params) -> Option<Vec<Point>> {
    if actual.len() < 2 || params.max_length <= params.epsilon {
        return None;
    }
    let distance = length(&actual);
    if distance <= params.epsilon {
        return None;
    }
    if distance - params.max_length > params.epsilon {
        clip_end(&mut actual, distance - params.max_length);
    }
    if actual.len() < 2 || actual.windows(2).any(|points| points[0] == points[1]) {
        return None;
    }
    actual[0] = params.seam_start;
    Some(actual)
}

pub(super) fn offset(original: &[Point], params: Params) -> Option<Vec<Point>> {
    if original.first() != Some(&params.seam_start) || params.max_length <= params.epsilon {
        return None;
    }
    let original_length = length(original);
    if original_length <= params.epsilon {
        return None;
    }
    let mut source_length = original_length.min(params.max_length);
    loop {
        let actual = from_source(original, original_length, source_length, params)?;
        if params.seam_start != params.seam_end
            && params.wipe_start == params.seam_end
            && backtracks(&actual, params.wipe_start, params.epsilon)
        {
            return None;
        }
        let actual_length = length(&actual);
        let exhausted = original_length - source_length <= params.epsilon;
        if actual_length + params.epsilon < params.max_length && !exhausted {
            let deficit = params.max_length - actual_length;
            let next = original_length.min(source_length + deficit.max(2.0 * params.epsilon));
            if next - source_length <= params.epsilon {
                return None;
            }
            source_length = next;
            continue;
        }
        return store(actual, params);
    }
}

fn from_source(
    original: &[Point],
    original_length: f64,
    source_length: f64,
    params: Params,
) -> Option<Vec<Point>> {
    let mut source = original.to_vec();
    let clip = original_length - source_length;
    if clip > params.epsilon {
        clip_end(&mut source, clip);
    }
    let mut wrapped = Vec::with_capacity(source.len() + 1);
    if params.seam_start == params.seam_end {
        let closing = original
            .iter()
            .rposition(|point| *point != params.seam_start)?;
        wrapped.push(original[closing]);
    } else {
        wrapped.push(params.seam_end);
    }
    wrapped.extend(source);
    let (mut offset, join) = build::polyline(&wrapped, params)?;
    if join == 0 || join >= offset.len() {
        return None;
    }
    offset.drain(..join);
    let mut actual = Vec::with_capacity(offset.len() + 1);
    actual.push(params.wipe_start);
    actual.extend(offset);
    if params.seam_start != params.seam_end
        && params.wipe_start != params.seam_start
        && params.wipe_start != params.seam_end
        && backtracks(&actual, params.wipe_start, params.epsilon)
        && !backtracks(&actual, params.seam_end, params.epsilon)
    {
        enter_at_projection(&mut actual, params.wipe_start);
    }
    Some(actual)
}

fn enter_at_projection(points: &mut Vec<Point>, start: Point) {
    let mut entry = 1;
    while entry + 1 < points.len() {
        let edge = Vector::delta(points[entry + 1], points[entry]);
        let projection = Vector::delta(start, points[entry]).dot(edge);
        if projection <= 0.0 {
            break;
        }
        if projection < edge.squared_norm() {
            points[entry] = (Vector::from_point(points[entry])
                + edge * (projection / edge.squared_norm()))
            .point();
            break;
        }
        entry += 1;
    }
    points.drain(1..entry);
}

pub(super) fn translated(
    original: &[Point],
    translation: Vector,
    params: Params,
) -> Option<Vec<Point>> {
    if translation.norm() <= params.epsilon || params.max_length <= params.epsilon {
        return None;
    }
    let mut actual = Vec::with_capacity(original.len() + 2);
    actual.push(params.wipe_start);
    append_translated(&mut actual, params.seam_start, translation);
    for &point in original {
        append_translated(&mut actual, point, translation);
    }
    if params.seam_start != params.seam_end
        && params.wipe_start == params.seam_end
        && backtracks(&actual, params.wipe_start, params.epsilon)
    {
        actual.truncate(1);
        append_translated(&mut actual, params.seam_end, translation);
    }
    store(actual, params)
}

fn append_translated(points: &mut Vec<Point>, point: Point, translation: Vector) {
    let point = (Vector::from_point(point) + translation).point();
    if Some(&point) != points.last() {
        points.push(point);
    }
}

pub(super) fn executable_length(points: &[Point], wipe_start: Point) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut result = Vector::delta(points[1], wipe_start).norm();
    for pair in points[1..].windows(2) {
        result += Vector::delta(pair[1], pair[0]).norm();
    }
    result
}
