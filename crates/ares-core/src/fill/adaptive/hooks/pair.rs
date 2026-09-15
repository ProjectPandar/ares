//! T-joint pair connection (`FillAdaptive.cpp:1150-1262`) — the anchor
//! line construction, could-connect gates, and the two-polyline merge
//! that determines the connected path direction.

use super::Joint;
use super::add_hook;
use super::create_offset_line;
use super::geometry::{
    distance_squared, dot, normalize, segment_line_intersection, segments_cross,
};
use crate::geometry::{Line, Point};

#[allow(clippy::too_many_arguments)]
pub(super) fn connect_pair(
    first: &mut Joint,
    nearest: &mut Joint,
    source: &[Line],
    working: &mut [Vec<Point>],
    merged_with: &mut [usize],
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
    if distance_squared(first_pt, nearest_pt) > hook_length_max * hook_length_max {
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
            if segments_cross(first_pt, nearest_pt, line.a, line.b).is_some() {
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
    // Connect (`:1209-1240`).
    if first.intersect_pl == nearest.intersect_pl {
        // Same polyline: a loop is being closed; swap the connection
        // points when the first joint sits at the back (`:1212-1213`).
        let (front_pt, back_pt) = if first.front {
            (first_pt, nearest_pt)
        } else {
            (nearest_pt, first_pt)
        };
        let points = &mut working[first.intersect_pl];
        points[0] = front_pt;
        let last = points.len() - 1;
        points[last] = back_pt;
        points.insert(0, back_pt);
    } else {
        // Different polylines: build the merged point list, optionally
        // trimming against each side's existing hook (`:1218-1228`).
        let mut l = Line::new(first_pt, nearest_pt);
        if let Some((nx, ny)) = normalize(
            source[first.closest_line].b.x() - source[first.closest_line].a.x(),
            source[first.closest_line].b.y() - source[first.closest_line].a.y(),
        ) {
            let sign = if first.left { 1.0 } else { -1.0 };
            let dx = (sign * scaled_trim_distance * -ny) as i64;
            let dy = (sign * scaled_trim_distance * nx) as i64;
            l = Line::new(
                Point::new(l.a.x() + dx, l.a.y() + dy),
                Point::new(l.b.x() + dx, l.b.y() + dy),
            );
        }
        let first_pl = first.intersect_pl;
        let nearest_pl = nearest.intersect_pl;
        // `trim_start/trim_end` (`:1218-1228`): `other_hook` yields None
        // unless the polyline is a just-hooked 3-pointer.
        let pt_start =
            other_hook(first, &working[first_pl]).and_then(|hook| line_line_point(hook, l));
        let pt_end =
            other_hook(nearest, &working[nearest_pl]).and_then(|hook| line_line_point(hook, l));

        let mut first_points = std::mem::take(&mut working[first_pl]);
        let second_points = &working[nearest_pl];
        if first.front {
            first_points.reverse();
        }
        if let Some(pt) = pt_start {
            first_points[0] = pt;
        }
        let last = first_points.len() - 1;
        first_points[last] = first_pt;
        first_points.push(nearest_pt);
        if nearest.front {
            first_points.extend_from_slice(&second_points[1..]);
        } else {
            // `second_points.rbegin() + 1` skips the LAST element.
            let len = second_points.len();
            let tail: Vec<Point> = second_points[..len - 1].iter().rev().copied().collect();
            first_points.extend(tail);
        }
        if let Some(pt) = pt_end {
            let last = first_points.len() - 1;
            first_points[last] = pt;
        }
        // Keep the polyline at the lower index slot (`:1232-1240`).
        if first_pl < nearest_pl {
            working[first_pl] = first_points;
            working[nearest_pl].clear();
            merged_with[nearest_pl] = first_pl;
            nearest.intersect_pl = first_pl;
        } else {
            working[nearest_pl] = first_points;
            working[first_pl].clear();
            merged_with[first_pl] = nearest_pl;
            first.intersect_pl = nearest_pl;
        }
    }
    first.used = true;
    nearest.used = true;
}

/// `Intersection::other_hook` (`FillAdaptive.cpp:600-606`): the hook
/// segment of a just-hooked 3-point polyline.
fn other_hook(joint: &Joint, points: &[Point]) -> Option<Line> {
    if points.len() >= 3 {
        Some(if joint.front {
            Line::new(points[1], points[2])
        } else {
            let len = points.len();
            Line::new(points[len - 2], points[len - 3])
        })
    } else {
        None
    }
}

fn line_line_point(hook: Line, l: Line) -> Option<Point> {
    segment_line_intersection(l, hook)
}

/// `update_merged_polyline_idx` (`:1049-1060`): follow the merge chain
/// to the surviving polyline slot (with path compression).
pub(super) fn resolve_merged(merged_with: &mut [usize], joint: &mut Joint) {
    let mut last = joint.intersect_pl;
    loop {
        let lower = merged_with[last];
        if lower == last {
            break;
        }
        last = lower;
    }
    merged_with[joint.intersect_pl] = last;
    joint.intersect_pl = last;
}

/// Borrow the `idx` and `other` joints in caller order (`first` is
/// always the current joint, not the lower index).
pub(super) fn split_at(group: &mut [Joint], idx: usize, other: usize) -> (&mut Joint, &mut Joint) {
    if idx < other {
        let (head, tail) = group.split_at_mut(other);
        (&mut head[idx], &mut tail[0])
    } else {
        let (head, tail) = group.split_at_mut(idx);
        (&mut tail[0], &mut head[other])
    }
}

/// `get_nearest_intersection` (`FillAdaptive.cpp:620-637`): with both
/// neighbors fresh take the closer one by projection along the closest
/// line; otherwise take next only if it is fresh. Neighbor freshness
/// uses the (possibly stale) polyline slot, matching upstream's lazy
/// `update_merged_polyline`.
pub(super) fn nearest_fresh(
    group: &[Joint],
    working: &[Vec<Point>],
    idx: usize,
    dir: (f64, f64),
) -> Option<usize> {
    if group.len() < 2 {
        return None;
    }
    let fresh = |joint: &Joint| !joint.used && !working[joint.intersect_pl].is_empty();
    let proj = |joint: &Joint| {
        dot(
            joint.intersect_point.x() as f64,
            joint.intersect_point.y() as f64,
            dir,
        )
    };
    let next_fresh = idx + 1 < group.len() && fresh(&group[idx + 1]);
    let prev_fresh = idx > 0 && fresh(&group[idx - 1]);
    let take_next = if idx == 0 {
        true
    } else if idx + 1 == group.len() {
        false
    } else if prev_fresh && next_fresh {
        proj(&group[idx + 1]) - proj(&group[idx]) < proj(&group[idx]) - proj(&group[idx - 1])
    } else {
        next_fresh
    };
    Some(if take_next { idx + 1 } else { idx - 1 })
}
