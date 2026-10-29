//! Horizontal-link selection plus the `vertical_outside` same-line
//! continuation for `traverse_graph_generate_polylines`
//! (`FillRectilinear.cpp:1485-1567`).

use crate::geometry::Point;

use super::super::perimeter::{emit_horizontal_arc, emit_vertical_arc, measure_horizontal_arc};
use super::super::segments::{LinkQuality, LinkType, RectilinearSlice};
use super::{intersection_point, is_inner, is_low, push_distinct, vertical_segment};

pub(super) struct Consumption<'a> {
    pub(super) vertical: &'a [Vec<bool>],
    pub(super) right: &'a [Vec<bool>],
}

#[expect(
    clippy::too_many_arguments,
    reason = "source traversal carries indexed graph state and output path"
)]
pub(super) fn connect_horizontal(
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
                && horizontal_available(
                    slice,
                    &consumption,
                    line,
                    intersection,
                    target_line,
                    target,
                    false,
                )
        });
    let right =
        horizontal_link(slice, line, intersection, true).filter(|&(target_line, target)| {
            horizontal_available(
                slice,
                &consumption,
                line,
                intersection,
                target_line,
                target,
                true,
            )
        });
    mark_horizontal_links(slice, consumed_right, line, intersection);
    let towards_right = match (left, right) {
        (None, None) => return None,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some((left_line, left_index)), Some((_right_line, right_index))) => {
            let left_length = measure_horizontal_arc(slice, left_line, left_index, intersection);
            let right_length = measure_horizontal_arc(slice, line, intersection, right_index);
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

/// `it->vertical_outside()` — the same-line Up/Down link continuing the
/// contour in the direction of travel, gated on not-yet-consumed segments
/// (`FillRectilinear.cpp:1525-1530`). Returns `(target, quality)`.
pub(super) fn vertical_outside(
    slice: &RectilinearSlice,
    consumed: &[Vec<bool>],
    line: usize,
    intersection: usize,
    going_up: bool,
) -> Option<(usize, LinkQuality)> {
    // `vertical_outside()` searches `left_vertical` (previous) before
    // `right_vertical` (next) — `FillRectilinear.cpp:267` — so ties resolve to
    // the contour-backward link.
    let wanted = if going_up {
        LinkType::Up
    } else {
        LinkType::Down
    };
    let item = slice.lines[line].intersections[intersection];
    let link = [item.previous, item.next]
        .into_iter()
        .flatten()
        .find(|&(_, kind, quality)| kind == wanted && quality != LinkQuality::Invalid)?;
    let target = link.0;
    let blocked = if going_up { target } else { target - 1 };
    if consumed[line][blocked] {
        return None;
    }
    Some((target, link.2))
}

#[expect(
    clippy::too_many_arguments,
    reason = "source continuation carries slice, consumption, link endpoints, and output buffers"
)]
/// Emits the `vertical_outside` continuation (`FillRectilinear.cpp:1531-1563`):
/// a Valid link follows the perimeter arc; a TooLong link ends the current
/// polyline and starts a new one at the target. Returns the target index.
pub(super) fn follow_vertical_outside(
    slice: &RectilinearSlice,
    consumed: &mut [bool],
    perimeter: &mut [Vec<bool>],
    line: usize,
    intersection: usize,
    target: usize,
    quality: LinkQuality,
    going_up: bool,
    points: &mut Vec<Point>,
    output: &mut Vec<crate::geometry::Polyline>,
) {
    push_distinct(points, intersection_point(slice, line, intersection));
    if quality == LinkQuality::Valid {
        // `FillRectilinear.cpp:1540-1541`: the arc follows the polygon's
        // contour direction, which is `going_up ? has_left_vertical_up()
        // : has_right_vertical_down()` — not the vertical travel direction.
        let item = slice.lines[line].intersections[intersection];
        let forward = if going_up {
            matches!(item.previous, Some((_, LinkType::Up, _)))
        } else {
            matches!(item.next, Some((_, LinkType::Down, _)))
        };
        emit_vertical_arc(slice, line, intersection, target, forward, points);
    } else {
        output.push(crate::geometry::Polyline::new(std::mem::take(points)));
        // FillRectilinear.cpp:1548-1551 starts the replacement polyline at
        // the target contour point before continuing the vertical run.
        *points = vec![intersection_point(slice, line, target)];
    }
    // `for (i = i_intersection; i < i_vertical; ++i)` / reverse — mark the
    // bypassed outer intersections so they cannot start a new path.
    let (low, high) = if going_up {
        (intersection, target)
    } else {
        (target, intersection)
    };
    for item in consumed.iter_mut().take(high).skip(low) {
        *item = true;
    }
    // `FillRectilinear.cpp:1557-1558`: mark `consumed_perimeter_right` on the
    // current intersection and the adjacent one along the scanline.
    perimeter[line][intersection] = true;
    let adjacent = if going_up {
        intersection + 1
    } else {
        intersection.wrapping_sub(1)
    };
    if let Some(item) = perimeter[line].get_mut(adjacent) {
        *item = true;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "source gate consumes both the current and target intersections"
)]
fn horizontal_available(
    slice: &RectilinearSlice,
    consumed: &Consumption<'_>,
    line: usize,
    intersection: usize,
    target_line: usize,
    target: usize,
    right: bool,
) -> bool {
    let target_intersection = slice.lines[target_line].intersections[target];
    // `FillRectilinear.cpp:521-527`: only follow a perimeter segment that
    // lands at the END of a vertical run — the outward-adjacent intersection
    // (`target-1` for a low target, `target+1` for high) must not be INNER.
    let outward = if is_low(target_intersection.kind) {
        target.checked_sub(1)
    } else {
        (target + 1 < slice.lines[target_line].intersections.len()).then_some(target + 1)
    };
    if outward.is_none_or(|index| is_inner(slice.lines[target_line].intersections[index].kind)) {
        return false;
    }
    let segment = vertical_segment(&slice.lines[target_line].intersections, target);
    // `FillRectilinear.cpp:532`: `side == Right ? it_this.consumed_perimeter_right
    // : it_other.consumed_perimeter_right`.
    let link_consumed = if right {
        consumed.right[line][intersection]
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
