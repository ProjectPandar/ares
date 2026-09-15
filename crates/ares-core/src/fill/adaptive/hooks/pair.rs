//! T-joint pair connection (`FillAdaptive.cpp:1150-1240`) — the anchor
//! line construction, could-connect gates, and the two-polyline merge
//! that determines the connected path direction.

use super::Joint;
use super::add_hook;
use super::create_offset_line;
use super::geometry::{
    distance_squared, ray_segment_hit, segment_line_intersection, segments_cross,
    shift_from_thick_line,
};
use crate::geometry::{Line, Point};

pub(super) fn connect_pair(
    first: &mut Joint,
    nearest: &mut Joint,
    source: &[Line],
    working: &mut [Vec<Point>],
    scaled_offset: f64,
    hook_length_max: f64,
    hook_length: f64,
    scaled_trim_distance: f64,
) {
    let anchor = create_offset_line(
        first.intersect_point,
        nearest.intersect_point,
        first.left,
        scaled_offset,
    );
    let Some(first_pt) = segment_line_intersection(source[first.intersect_line], anchor) else {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    };
    let Some(nearest_pt) = segment_line_intersection(source[nearest.intersect_line], anchor) else {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    };
    let d2 = distance_squared(first_pt, nearest_pt);
    if d2 > hook_length_max * hook_length_max {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    }
    // No third source line may cross the anchor segment.
    for (index, line) in source.iter().enumerate() {
        if index != first.intersect_line && index != nearest.intersect_line {
            if let Some(_) = segments_cross(first_pt, nearest_pt, line.a, line.b) {
                add_hook(
                    first,
                    source,
                    working,
                    scaled_offset,
                    hook_length,
                    scaled_trim_distance,
                );
                first.used = true;
                return;
            }
        }
    }
    // Connect: same-polyline loop close or two-polyline merge.
    if first.intersect_pl == nearest.intersect_pl {
        let points = &mut working[first.intersect_pl];
        if !first.front {
            points[0] = first_pt;
            let last = points.len() - 1;
            points[last] = nearest_pt;
            points.insert(0, nearest_pt);
        } else {
            points[0] = first_pt;
            let last = points.len() - 1;
            points[last] = nearest_pt;
            points.insert(0, nearest_pt);
        }
    } else {
        let mut first_points = std::mem::take(&mut working[first.intersect_pl]);
        let mut second_points = std::mem::take(&mut working[nearest.intersect_pl]);
        if first.front {
            first_points.reverse();
        }
        let last = first_points.len() - 1;
        first_points[last] = first_pt;
        first_points.push(nearest_pt);
        if nearest.front {
            first_points.extend_from_slice(&second_points[1..]);
        } else {
            let tail: Vec<Point> = second_points[1..].iter().rev().copied().collect();
            first_points.extend(tail);
        }
        // Keep the lower index slot (`:1232-1240`).
        if first.intersect_pl < nearest.intersect_pl {
            working[first.intersect_pl] = first_points;
            second_points.clear();
            working[nearest.intersect_pl] = second_points;
            nearest.intersect_pl = first.intersect_pl;
        } else {
            working[nearest.intersect_pl] = first_points;
            first_points_copy_clear(&mut working[first.intersect_pl]);
            working[first.intersect_pl] = Vec::new();
            first.intersect_pl = nearest.intersect_pl;
        }
    }
    first.used = true;
    nearest.used = true;
}

pub(super) fn first_points_copy_clear(_slot: &mut Vec<Point>) {}

pub(super) fn split_at(group: &mut [Joint], idx: usize, other: usize) -> (&mut Joint, &mut Joint) {
    let (lo, hi) = if idx < other {
        (idx, other)
    } else {
        (other, idx)
    };
    let (head, tail) = group.split_at_mut(hi);
    (&mut head[lo], &mut tail[0])
}

pub(super) fn nearest_fresh(group: &[Joint], idx: usize) -> Option<usize> {
    if group.len() < 2 {
        return None;
    }
    let prev_fresh = idx > 0 && !group[idx - 1].used && !group[idx - 1].used;
    let next_fresh = idx + 1 < group.len() && !group[idx + 1].used;
    let take_next = if idx == 0 {
        true
    } else if idx + 1 == group.len() {
        false
    } else if prev_fresh && next_fresh {
        // closer neighbor by projection order = the sorted distance
        false
    } else {
        next_fresh
    };
    Some(if take_next { idx + 1 } else { idx - 1 })
}
