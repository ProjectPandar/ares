mod links;

use crate::geometry::{Point, Polyline};

use super::segments::{IntersectionKind, RectilinearSlice, SegmentIntersection};
use links::{connect_horizontal, follow_vertical_outside, vertical_outside};

pub(super) fn generate(slice: &RectilinearSlice, consistent_pattern: bool) -> Vec<Polyline> {
    let mut consumed_vertical = slice
        .lines
        .iter()
        .map(|line| vec![false; line.intersections.len()])
        .collect::<Vec<_>>();
    let mut consumed_right = consumed_vertical.clone();
    for (line_index, line) in slice.lines.iter().enumerate() {
        for (index, consumed) in consumed_vertical[line_index]
            .iter_mut()
            .enumerate()
            .take(line.intersections.len().saturating_sub(1))
        {
            if line.intersections[index].kind == IntersectionKind::OuterLow
                && line.intersections[index + 1].kind == IntersectionKind::OuterHigh
            {
                *consumed = true;
            }
        }
    }

    let mut output = Vec::new();
    while let Some((mut line_index, mut intersection_index)) =
        next_start(slice, &consumed_vertical, consistent_pattern)
    {
        let mut points = vec![intersection_point(slice, line_index, intersection_index)];
        loop {
            let going_up = is_low(slice.lines[line_index].intersections[intersection_index].kind);
            let try_connect = consume_vertical_run(
                slice,
                &mut consumed_vertical,
                line_index,
                &mut intersection_index,
                going_up,
            );
            if try_connect
                && let Some((next_line, next_intersection)) = connect_horizontal(
                    slice,
                    &consumed_vertical,
                    &mut consumed_right,
                    line_index,
                    intersection_index,
                    &mut points,
                    consistent_pattern,
                )
            {
                line_index = next_line;
                intersection_index = next_intersection;
                continue;
            }
            if try_connect
                && let Some((target, quality)) = vertical_outside(
                    slice,
                    &consumed_vertical,
                    line_index,
                    intersection_index,
                    going_up,
                )
            {
                // `vertical_outside` (`FillRectilinear.cpp:1525-1563`): a
                // same-line contour link continues the path before finishing.
                follow_vertical_outside(
                    slice,
                    &mut consumed_vertical[line_index],
                    line_index,
                    intersection_index,
                    target,
                    quality,
                    going_up,
                    &mut points,
                    &mut output,
                );
                intersection_index = target;
                continue;
            }
            if try_connect {
                intersection_index = outer_intersection(intersection_index, going_up);
            }
            push_distinct(
                &mut points,
                intersection_point(slice, line_index, intersection_index),
            );
            break;
        }
        if points.len() > 1 {
            output.push(Polyline::new(points));
        }
    }
    output
}

fn next_start(
    slice: &RectilinearSlice,
    consumed: &[Vec<bool>],
    consistent_pattern: bool,
) -> Option<(usize, usize)> {
    for (line_index, line) in slice.lines.iter().enumerate() {
        let forward = !consistent_pattern || line_index % 2 == 0;
        for offset in 0..line.intersections.len() {
            let index = if forward {
                offset
            } else {
                line.intersections.len() - 1 - offset
            };
            let item = line.intersections[index];
            if is_outer(item.kind) && !vertical_consumed(line, &consumed[line_index], index) {
                return Some((line_index, index));
            }
        }
    }
    None
}

fn consume_vertical_run(
    slice: &RectilinearSlice,
    consumed: &mut [Vec<bool>],
    line_index: usize,
    intersection_index: &mut usize,
    going_up: bool,
) -> bool {
    let line = &slice.lines[line_index];
    if going_up {
        if is_inner(line.intersections[*intersection_index].kind) {
            *intersection_index -= 1;
        }
        loop {
            consumed[line_index][*intersection_index] = true;
            *intersection_index += 1;
            if line.intersections[*intersection_index].kind == IntersectionKind::OuterHigh {
                break;
            }
        }
        if is_inner(line.intersections[*intersection_index - 1].kind) {
            *intersection_index -= 1;
            return true;
        }
    } else {
        if is_inner(line.intersections[*intersection_index].kind) {
            consumed[line_index][*intersection_index] = true;
        }
        loop {
            *intersection_index -= 1;
            consumed[line_index][*intersection_index] = true;
            if line.intersections[*intersection_index].kind == IntersectionKind::OuterLow {
                break;
            }
        }
        if is_inner(line.intersections[*intersection_index + 1].kind) {
            *intersection_index += 1;
            return true;
        }
    }
    false
}

fn vertical_consumed(
    line: &super::segments::SegmentedLine,
    consumed: &[bool],
    intersection: usize,
) -> bool {
    consumed[vertical_segment(&line.intersections, intersection)]
}

pub(super) fn vertical_segment(intersections: &[SegmentIntersection], index: usize) -> usize {
    if is_low(intersections[index].kind) {
        index
    } else {
        index - 1
    }
}

const fn outer_intersection(index: usize, going_up: bool) -> usize {
    if going_up { index + 1 } else { index - 1 }
}

pub(super) fn intersection_point(
    slice: &RectilinearSlice,
    line: usize,
    intersection: usize,
) -> Point {
    Point::new(
        slice.lines[line].x,
        slice.lines[line].intersections[intersection].point.y(),
    )
}

pub(super) fn push_distinct(points: &mut Vec<Point>, point: Point) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}

pub(super) const fn is_low(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::InnerLow
    )
}

const fn is_outer(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::OuterHigh
    )
}

pub(super) const fn is_inner(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::InnerLow | IntersectionKind::InnerHigh
    )
}
