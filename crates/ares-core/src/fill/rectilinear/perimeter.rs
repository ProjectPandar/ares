use crate::geometry::{Point, Polygon};

use super::segments::{IntersectionKind, RectilinearSlice};

pub(crate) const fn directed_segment_distance(
    point_count: usize,
    first: usize,
    second: usize,
    forward: bool,
) -> usize {
    if forward {
        if second >= first {
            second - first
        } else {
            second + point_count - first
        }
    } else if first >= second {
        first - second
    } else {
        first + point_count - second
    }
}

pub(crate) fn contour_segment_length(
    polygon: &Polygon,
    first_segment: usize,
    first: Point,
    second_segment: usize,
    second: Point,
) -> f64 {
    let points = polygon.points();
    let mut previous = first;
    let mut length = 0.0;
    let count = directed_segment_distance(points.len(), first_segment, second_segment, true);
    for offset in 0..count {
        let point = points[(first_segment + offset) % points.len()];
        length += point_distance(previous, point);
        previous = point;
    }
    length + point_distance(previous, second)
}

pub(crate) fn append_contour_segment(
    output: &mut Vec<Point>,
    polygon: &Polygon,
    first_segment: usize,
    second_segment: usize,
    forward: bool,
) {
    let points = polygon.points();
    if forward {
        let count = directed_segment_distance(points.len(), first_segment, second_segment, true);
        output.extend((0..count).map(|offset| points[(first_segment + offset) % points.len()]));
    } else {
        let count = directed_segment_distance(points.len(), first_segment, second_segment, false);
        output.extend(
            (0..count)
                .map(|offset| points[(first_segment + points.len() - 1 - offset) % points.len()]),
        );
    }
}

pub(crate) fn measure_horizontal_arc(
    slice: &RectilinearSlice,
    left_line: usize,
    left_intersection: usize,
    right_intersection: usize,
) -> f64 {
    let left = slice.lines[left_line].intersections[left_intersection];
    let right = slice.lines[left_line + 1].intersections[right_intersection];
    let polygon = &slice.contours[left.contour_index].polygon;
    if is_low(left.kind) {
        contour_segment_length(
            polygon,
            left.segment_index,
            left.point,
            right.segment_index,
            right.point,
        )
    } else {
        contour_segment_length(
            polygon,
            right.segment_index,
            right.point,
            left.segment_index,
            left.point,
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "source emitter identifies two indexed intersections plus direction and output"
)]
pub(crate) fn emit_horizontal_arc(
    slice: &RectilinearSlice,
    line: usize,
    intersection: usize,
    other_intersection: usize,
    towards_next: bool,
    output: &mut Vec<Point>,
) {
    let other_line = if towards_next { line + 1 } else { line - 1 };
    let first = slice.lines[line].intersections[intersection];
    let second = slice.lines[other_line].intersections[other_intersection];
    let polygon = &slice.contours[first.contour_index].polygon;
    append_contour_segment(
        output,
        polygon,
        first.segment_index,
        second.segment_index,
        is_low(first.kind) == towards_next,
    );
    output.push(second.point);
}

pub(crate) fn measure_vertical_arc(
    slice: &RectilinearSlice,
    line: usize,
    first_intersection: usize,
    second_intersection: usize,
    forward: bool,
) -> f64 {
    let first = slice.lines[line].intersections[first_intersection];
    let second = slice.lines[line].intersections[second_intersection];
    let polygon = &slice.contours[first.contour_index].polygon;
    if forward {
        contour_segment_length(
            polygon,
            first.segment_index,
            first.point,
            second.segment_index,
            second.point,
        )
    } else {
        contour_segment_length(
            polygon,
            second.segment_index,
            second.point,
            first.segment_index,
            first.point,
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "source emitter identifies two indexed intersections plus direction and output"
)]
pub(crate) fn emit_vertical_arc(
    slice: &RectilinearSlice,
    line: usize,
    first_intersection: usize,
    second_intersection: usize,
    forward: bool,
    output: &mut Vec<Point>,
) {
    let first = slice.lines[line].intersections[first_intersection];
    let second = slice.lines[line].intersections[second_intersection];
    append_contour_segment(
        output,
        &slice.contours[first.contour_index].polygon,
        first.segment_index,
        second.segment_index,
        forward,
    );
    output.push(second.point);
}

fn point_distance(first: Point, second: Point) -> f64 {
    let x = (first.x() - second.x()) as f64;
    let y = (first.y() - second.y()) as f64;
    (x * x + y * y).sqrt()
}

const fn is_low(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::InnerLow
    )
}
