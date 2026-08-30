use crate::geometry::{Point, Polyline};

use super::{
    perimeter::emit_horizontal_arc,
    segments::{IntersectionKind, LinkQuality, LinkType, RectilinearSlice, SegmentIntersection},
};

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

#[expect(
    clippy::too_many_arguments,
    reason = "source traversal carries indexed graph state and output path"
)]
fn connect_horizontal(
    slice: &RectilinearSlice,
    consumed_vertical: &[Vec<bool>],
    consumed_right: &mut [Vec<bool>],
    line: usize,
    intersection: usize,
    points: &mut Vec<Point>,
    consistent_pattern: bool,
) -> Option<(usize, usize)> {
    let consumption = Consumption {
        vertical: consumed_vertical,
        right: consumed_right,
    };
    let left =
        horizontal_link(slice, line, intersection, false).filter(|&(target_line, target)| {
            !consistent_pattern
                && horizontal_available(slice, &consumption, target_line, target, false)
        });
    let right =
        horizontal_link(slice, line, intersection, true).filter(|&(target_line, target)| {
            horizontal_available(slice, &consumption, target_line, target, true)
        });
    mark_horizontal_links(slice, consumed_right, line, intersection);
    let towards_right = match (left, right) {
        (None, None) => return None,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some((left_line, left_index)), Some((_right_line, right_index))) => {
            let left_length = super::perimeter::measure_horizontal_arc(
                slice,
                left_line,
                left_index,
                intersection,
            );
            let right_length =
                super::perimeter::measure_horizontal_arc(slice, line, intersection, right_index);
            right_length < left_length
        }
    };
    let (target_line, target) = if towards_right {
        right.expect("right link is available")
    } else {
        left.expect("left link is available")
    };
    push_distinct(points, intersection_point(slice, line, intersection));
    emit_horizontal_arc(slice, line, intersection, target, towards_right, points);
    Some((target_line, target))
}

fn horizontal_link(
    slice: &RectilinearSlice,
    line: usize,
    intersection: usize,
    right: bool,
) -> Option<(usize, usize)> {
    let link = if right {
        slice.lines[line].intersections[intersection].next
    } else {
        slice.lines[line].intersections[intersection].previous
    }?;
    if link.1 != LinkType::Horizontal || link.2 != LinkQuality::Valid {
        return None;
    }
    Some((if right { line + 1 } else { line - 1 }, link.0))
}

struct Consumption<'a> {
    vertical: &'a [Vec<bool>],
    right: &'a [Vec<bool>],
}

fn horizontal_available(
    slice: &RectilinearSlice,
    consumed: &Consumption<'_>,
    target_line: usize,
    target: usize,
    right: bool,
) -> bool {
    let segment = vertical_segment(&slice.lines[target_line].intersections, target);
    let link_consumed = if right {
        consumed.right[target_line - 1][slice.lines[target_line].intersections[target]
            .previous
            .unwrap()
            .0]
    } else {
        consumed.right[target_line][target]
    };
    !consumed.vertical[target_line][segment] && !link_consumed
}

fn mark_horizontal_links(
    slice: &RectilinearSlice,
    consumed_right: &mut [Vec<bool>],
    line: usize,
    intersection: usize,
) {
    if let Some((target, LinkType::Horizontal, _)) =
        slice.lines[line].intersections[intersection].previous
    {
        consumed_right[line - 1][target] = true;
    }
    if matches!(
        slice.lines[line].intersections[intersection].next,
        Some((_, LinkType::Horizontal, _))
    ) {
        consumed_right[line][intersection] = true;
    }
}

fn vertical_consumed(
    line: &super::segments::SegmentedLine,
    consumed: &[bool],
    intersection: usize,
) -> bool {
    consumed[vertical_segment(&line.intersections, intersection)]
}

fn vertical_segment(intersections: &[SegmentIntersection], index: usize) -> usize {
    if is_low(intersections[index].kind) {
        index
    } else {
        index - 1
    }
}

const fn outer_intersection(index: usize, going_up: bool) -> usize {
    if going_up { index + 1 } else { index - 1 }
}

fn intersection_point(slice: &RectilinearSlice, line: usize, intersection: usize) -> Point {
    Point::new(
        slice.lines[line].x,
        slice.lines[line].intersections[intersection].point.y(),
    )
}

fn push_distinct(points: &mut Vec<Point>, point: Point) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}

const fn is_low(kind: IntersectionKind) -> bool {
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

const fn is_inner(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::InnerLow | IntersectionKind::InnerHigh
    )
}
