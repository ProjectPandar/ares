//! Curled-perimeter geometry (`SupportSpotsGenerator.cpp:102-135`,
//! `ExtrusionProcessor.hpp:219-278, 375-420`): the curl-height formula,
//! the windowed curvature, and the artificial distance with its
//! projected-length influence box.

use super::CurledLine;
use crate::geometry::{CoordinateScale, LineDistanceTree, Point};

const MALFORMATION_DISTANCE_FACTORS: (f32, f32) = (0.2, 1.1);
const MAX_CURLED_HEIGHT_FACTOR: f32 = 10.0;
const CURVATURE_WINDOWS: [f64; 3] = [3.0, 9.0, 16.0];

/// `estimate_curled_up_height` (`SupportSpotsGenerator.cpp:102-135`):
/// the decay toward the layer below, the swelling and convex-turn
/// tension of the malformation band, capped by the max height factor.
pub(super) fn estimate_curled_up_height(
    distance: f32,
    curvature: f32,
    layer_height: f32,
    flow_width: f32,
    prev_line_curled_height: f32,
) -> f32 {
    let mut curled_up_height = 0.0_f32;
    if distance.abs() < 3.0 * flow_width {
        curled_up_height = (prev_line_curled_height - layer_height * 0.75).max(0.0);
    }
    let low = MALFORMATION_DISTANCE_FACTORS.0 * flow_width;
    let high = MALFORMATION_DISTANCE_FACTORS.1 * flow_width;
    if distance > low && distance < high {
        let curling_section = distance;
        let swelling_radius = (layer_height + curling_section) / 2.0;
        curled_up_height += ((swelling_radius - layer_height) / 2.0).max(0.0);
        if curvature > 0.01 {
            let radius = 1.0 / curvature;
            let curling_t = (radius / 100.0_f32).sqrt();
            let b = curling_t * flow_width;
            let c = (curling_section * curling_section - b * b).max(0.0).sqrt();
            curled_up_height += c;
        }
        curled_up_height = curled_up_height.min(MAX_CURLED_HEIGHT_FACTOR * layer_height);
    }
    curled_up_height
}

/// The artificial curl distance for one point pair
/// (`ExtrusionProcessor.hpp:382-420`): lines within `10·width` of the
/// segment midpoint contribute, segments longer than 2mm require the
/// curled lines to span 40% of their direction inside the influence
/// box.
pub(crate) fn artificial_distance(
    curled: &[CurledLine],
    current: (f64, f64),
    next: (f64, f64),
    width: f32,
    height: f32,
) -> f32 {
    let dist_limit = 10.0 * f64::from(width);
    let middle = ((current.0 + next.0) * 0.5, (current.1 + next.1) * 0.5);
    let length = (next.0 - current.0).hypot(next.1 - current.1);
    let in_range: Vec<&CurledLine> = curled
        .iter()
        .filter(|line| {
            f64::from(point_segment_distance(
                middle,
                (line.x0, line.y0),
                (line.x1, line.y1),
            )) < dist_limit
        })
        .collect();
    if in_range.is_empty() {
        return 0.0;
    }
    if length > 2.0 && projected_length_sum(&in_range, current, next, dist_limit) < 0.4 * length {
        return 0.0;
    }
    in_range
        .iter()
        .map(|line| {
            let distance_from_curled =
                point_segment_distance(middle, (line.x0, line.y0), (line.x1, line.y1));
            let ratio = 1.0 - distance_from_curled / dist_limit as f32;
            width * ratio * ratio * (line.curled_height / (height * 10.0))
        })
        .fold(0.0_f32, f32::max)
}

/// Sum of the curled lines' extents projected onto the segment direction
/// inside the perpendicular influence box (`intersection_ln` +
/// `projected_length`, `ExtrusionProcessor.hpp:396-413`).
fn projected_length_sum(
    lines: &[&CurledLine],
    current: (f64, f64),
    next: (f64, f64),
    dist_limit: f64,
) -> f64 {
    let dx = next.0 - current.0;
    let dy = next.1 - current.1;
    let length = dx.hypot(dy);
    if length <= 0.0 {
        return 0.0;
    }
    let dir = (dx / length, dy / length);
    let right = (-dir.1, dir.0);
    let mut sum = 0.0;
    for line in lines {
        // Clip the curled segment to the local-frame box
        // u ∈ [0, length], v ∈ [-dist_limit, dist_limit].
        let to_local = |point: (f64, f64)| {
            let rx = point.0 - current.0;
            let ry = point.1 - current.1;
            (rx * dir.0 + ry * dir.1, rx * right.0 + ry * right.1)
        };
        let (u0, v0) = to_local((line.x0, line.y0));
        let (u1, v1) = to_local((line.x1, line.y1));
        let mut t_enter = 0.0_f64;
        let mut t_exit = 1.0_f64;
        let clipped = clip_axis(u0, u1 - u0, 0.0, length, &mut t_enter, &mut t_exit)
            && clip_axis(
                v0,
                v1 - v0,
                -dist_limit,
                dist_limit,
                &mut t_enter,
                &mut t_exit,
            )
            && t_enter <= t_exit;
        if clipped {
            sum += (u1 - u0).abs() * (t_exit - t_enter);
        }
    }
    sum
}

/// Liang-Barsky clip of `origin + t·delta` to `[low, high]`: low bounds
/// enter on positive deltas, high bounds exit; parallel segments reject
/// when outside the bound.
fn clip_axis(
    origin: f64,
    delta: f64,
    low: f64,
    high: f64,
    t_enter: &mut f64,
    t_exit: &mut f64,
) -> bool {
    for (bound, is_low) in [(low, true), (high, false)] {
        if delta == 0.0 {
            if (is_low && origin < bound) || (!is_low && origin > bound) {
                return false;
            }
        } else {
            let t = (bound - origin) / delta;
            if (is_low && delta > 0.0) || (!is_low && delta < 0.0) {
                *t_enter = (*t_enter).max(t);
            } else {
                *t_exit = (*t_exit).min(t);
            }
        }
    }
    true
}

/// Per-point curvature over the 3/9/16mm windows
/// (`ExtrusionProcessor.hpp:219-278`): for each window, the angle
/// between the half-window back and front tangents divided by the
/// window size; the largest magnitude wins.
pub(super) fn curvatures(points: &[(f64, f64)]) -> Vec<f32> {
    let count = points.len();
    let mut result = vec![0.0_f32; count];
    if count < 3 {
        return result;
    }
    let looped = (points[0].0 - points[count - 1].0).abs() < 1e-9
        && (points[0].1 - points[count - 1].1).abs() < 1e-9;
    let mut predecessor_length = vec![0.0_f64; count];
    for index in 0..count {
        let previous = prev_index(index, count, looped);
        predecessor_length[index] =
            (points[previous].0 - points[index].0).hypot(points[previous].1 - points[index].1);
    }
    for &window in CURVATURE_WINDOWS.iter() {
        for index in 0..count {
            let back = walk(
                points,
                &predecessor_length,
                index,
                window * 0.5,
                true,
                looped,
            );
            let front = walk(
                points,
                &predecessor_length,
                index,
                window * 0.5,
                false,
                looped,
            );
            let current = points[index];
            let back_angle = (current.0 - back.0, current.1 - back.1);
            let front_angle = (front.0 - current.0, front.1 - current.1);
            let cross = back_angle.0 * front_angle.1 - back_angle.1 * front_angle.0;
            let dot = back_angle.0 * front_angle.0 + back_angle.1 * front_angle.1;
            let new_curvature = (cross.atan2(dot) as f32) / window as f32;
            if result[index].abs() < new_curvature.abs() {
                result[index] = new_curvature;
            }
        }
    }
    result
}

/// Interpolated position half a window away along the polyline, walking
/// backward or forward and cutting the final step at the exact distance
/// (`ExtrusionProcessor.hpp:232-272`). Upstream reads the NEIGHBOURING
/// segment length for the fit check (`distances_for_curvature[prev(back)]`
/// on the back walk and `[front]` on the front walk) while interpolating
/// along the traversed direction — replicated verbatim.
fn walk(
    points: &[(f64, f64)],
    predecessor_length: &[f64],
    start: usize,
    half_window: f64,
    backwards: bool,
    looped: bool,
) -> (f64, f64) {
    let count = points.len();
    let mut position = points[start];
    let mut index = start;
    let mut distance = 0.0_f64;
    loop {
        let neighbor = if backwards {
            prev_index(index, count, looped)
        } else {
            next_index(index, count, looped)
        };
        if neighbor == index || distance >= half_window {
            break;
        }
        let line_dist = if backwards {
            predecessor_length[prev_index(index, count, looped)]
        } else {
            predecessor_length[index]
        };
        if distance + line_dist > half_window {
            let remaining = half_window - distance;
            let delta = (
                points[neighbor].0 - position.0,
                points[neighbor].1 - position.1,
            );
            let norm = delta.0.hypot(delta.1);
            if norm > 0.0 {
                position = (
                    position.0 + remaining * delta.0 / norm,
                    position.1 + remaining * delta.1 / norm,
                );
            }
            break;
        }
        distance += line_dist;
        index = neighbor;
        position = points[index];
    }
    position
}

fn prev_index(index: usize, count: usize, looped: bool) -> usize {
    if index > 0 {
        index - 1
    } else if looped {
        count - 1
    } else {
        0
    }
}

fn next_index(index: usize, count: usize, looped: bool) -> usize {
    if index + 1 < count {
        index + 1
    } else if looped {
        0
    } else {
        count - 1
    }
}

pub(super) fn point_segment_distance(point: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= 0.0 {
        return (point.0 - a.0).hypot(point.1 - a.1) as f32;
    }
    let t = (((point.0 - a.0) * dx + (point.1 - a.1) * dy) / length_squared).clamp(0.0, 1.0);
    (point.0 - (a.0 + t * dx)).hypot(point.1 - (a.1 + t * dy)) as f32
}

pub(super) fn signed_distance(
    tree: &LineDistanceTree<'_>,
    scale: CoordinateScale,
    point: (f64, f64),
) -> f32 {
    let nearest = tree
        .nearest_f32([point.0 as f32, point.1 as f32], scale)
        .expect("a nonempty boundary has a nearest line");
    let scaled = Point::new(
        (point.0 / scale.factor()) as i64,
        (point.1 / scale.factor()) as i64,
    );
    tree.outside(scaled) as f32 * nearest.squared_distance.sqrt()
}
