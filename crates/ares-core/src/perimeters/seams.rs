use super::{PerimeterOptions, PerimeterRole, SeamPosition};
use crate::Point2;

pub(super) fn position_loop(
    points: Vec<Point2>,
    role: PerimeterRole,
    options: PerimeterOptions,
    stagger_depth_mm: f64,
) -> Vec<Point2> {
    let points = rotate_for_seam_position(points, options.seam_position());
    if role == PerimeterRole::Internal
        && options.staggered_inner_seams()
        && options.seam_position() == SeamPosition::Back
    {
        return stagger_along_loop(points, stagger_depth_mm.max(options.internal_line_width()));
    }
    points
}

fn rotate_for_seam_position(mut points: Vec<Point2>, seam_position: SeamPosition) -> Vec<Point2> {
    if seam_position != SeamPosition::Back {
        return points;
    }
    if let Some(index) = first_max_y_index(&points) {
        points.rotate_left(index);
    }
    points
}

fn first_max_y_index(points: &[Point2]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        if best.is_none_or(|best_index| point.y() > points[best_index].y()) {
            best = Some(index);
        }
    }
    best
}

fn stagger_along_loop(points: Vec<Point2>, distance_mm: f64) -> Vec<Point2> {
    if points.len() < 2 || distance_mm <= 0.0 {
        return points;
    }
    let length = closed_length(&points);
    if length <= f64::EPSILON {
        return points;
    }
    let mut remaining = distance_mm % length;
    if remaining <= f64::EPSILON {
        return points;
    }
    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        let segment = distance(start, end);
        if segment <= f64::EPSILON {
            continue;
        }
        if remaining < segment {
            return split_at_segment_point(
                points,
                index,
                point_between(start, end, remaining / segment),
            );
        }
        if (remaining - segment).abs() <= f64::EPSILON {
            let mut rotated = points;
            let next_index = (index + 1) % rotated.len();
            rotated.rotate_left(next_index);
            return rotated;
        }
        remaining -= segment;
    }
    points
}

fn split_at_segment_point(
    points: Vec<Point2>,
    segment_start_index: usize,
    seam: Point2,
) -> Vec<Point2> {
    let mut shifted = Vec::with_capacity(points.len() + 1);
    shifted.push(seam);
    let mut index = (segment_start_index + 1) % points.len();
    while index != segment_start_index {
        shifted.push(points[index]);
        index = (index + 1) % points.len();
    }
    shifted.push(points[segment_start_index]);
    shifted
}

fn closed_length(points: &[Point2]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(start, end)| distance(*start, *end))
        .sum()
}

fn point_between(start: Point2, end: Point2, ratio: f64) -> Point2 {
    Point2::new(
        start.x() + (end.x() - start.x()) * ratio,
        start.y() + (end.y() - start.y()) * ratio,
    )
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}
