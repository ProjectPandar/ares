//! Curled-perimeter geometry (`SupportSpotsGenerator.cpp:102-135`,
//! `ExtrusionProcessor.hpp:219-278, 375-420`): the curl-height formula,
//! the windowed curvature, and the artificial distance with its
//! projected-length influence box.

use super::super::sections::scale_trunc;
use super::CurledLine;
use crate::geometry::{CoordinateScale, LineDistanceTree};

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
    [current, next]: [(f64, f64); 2],
    width: f32,
    height: f32,
    scale: CoordinateScale,
) -> f32 {
    let dist_limit = 10.0 * f64::from(width);
    let middle = ((current.0 + next.0) * 0.5, (current.1 + next.1) * 0.5);
    // Upstream quantizes the query midpoint and the radius into the
    // scaled coordinate domain (`Point::new_scale(middle)`,
    // `scale_(dist_limit)`, `ExtrusionProcessor.hpp:381-382`).
    let scaled_middle = (scale_trunc(middle.0, scale), scale_trunc(middle.1, scale));
    let scaled_radius = scale_trunc(dist_limit, scale);
    let length = (next.0 - current.0).hypot(next.1 - current.1);
    // `all_lines_in_radius` filters on squared segment distance in the
    // scaled-int domain (`AABBTreeLines.hpp:222-236`).
    let in_range: Vec<&CurledLine> = curled
        .iter()
        .filter(|line| {
            segment_distance_scaled(scaled_middle, (line.x0, line.y0), (line.x1, line.y1))
                <= scaled_radius as f64 * scaled_radius as f64
        })
        .collect();
    if in_range.is_empty() {
        return 0.0;
    }
    if length > 2.0
        && projected_length_sum(&in_range, current, next, dist_limit, scale) < 0.4 * length
    {
        return 0.0;
    }
    in_range
        .iter()
        .map(|line| {
            // `float distance_from_curled = unscaled(distance_to(...))`:
            // int-domain distance, unscaled, stored float.
            let distance_from_curled =
                (segment_distance_scaled(scaled_middle, (line.x0, line.y0), (line.x1, line.y1))
                    .sqrt()
                    * scale.factor()) as f32;
            // `path.width * (1 - d/limit)^2 * (h/(height*10))` —
            // `d/dist_limit` promotes to double, so the product is f64
            // before the float store (`ExtrusionProcessor.hpp:416-419`).
            (f64::from(width)
                * (1.0 - f64::from(distance_from_curled) / dist_limit)
                * (1.0 - f64::from(distance_from_curled) / dist_limit)
                * f64::from(line.curled_height / (height * 10.0))) as f32
        })
        .fold(0.0_f32, f32::max)
}

/// `line_alg::distance_to` squared: closest point of the segment to
/// `point`, all in scaled-int coordinates, returning the squared
/// distance in scaled units.
fn segment_distance_scaled(point: (i64, i64), a: (i64, i64), b: (i64, i64)) -> f64 {
    let dx = (b.0 - a.0) as f64;
    let dy = (b.1 - a.1) as f64;
    let length_squared = dx * dx + dy * dy;
    let (px, py) = (point.0 as f64, point.1 as f64);
    if length_squared <= 0.0 {
        return (px - a.0 as f64).powi(2) + (py - a.1 as f64).powi(2);
    }
    let t = (((px - a.0 as f64) * dx + (py - a.1 as f64) * dy) / length_squared).clamp(0.0, 1.0);
    let dx = px - (a.0 as f64 + t * dx);
    let dy = py - (a.1 as f64 + t * dy);
    dx * dx + dy * dy
}

/// Sum of the curled lines' extents projected onto the segment direction
/// inside the perpendicular influence box (`intersection_ln` +
/// `projected_length`, `ExtrusionProcessor.hpp:396-413`).
fn projected_length_sum(
    lines: &[&CurledLine],
    current: (f64, f64),
    next: (f64, f64),
    dist_limit: f64,
    scale: CoordinateScale,
) -> f64 {
    let dx = next.0 - current.0;
    let dy = next.1 - current.1;
    let length = dx.hypot(dy);
    if length <= 0.0 {
        return 0.0;
    }
    let dir = (dx / length, dy / length);
    let right = (-dir.1, dir.0);
    let factor = scale.factor();
    let mut sum = 0.0;
    for line in lines {
        // Clip the curled segment to the local-frame box
        // u ∈ [0, length], v ∈ [-dist_limit, dist_limit]. The curled
        // endpoints are scaled ints; unscale back to mm first.
        let to_local = |point: (i64, i64)| {
            let rx = point.0 as f64 * factor - current.0;
            let ry = point.1 as f64 * factor - current.1;
            (rx * dir.0 + ry * dir.1, rx * right.0 + ry * right.1)
        };
        let (u0, v0) = to_local((line.x0, line.y0));
        let (u1, v1) = to_local((line.x1, line.y1));
        let mut t_enter = 0.0_f64;
        let mut t_exit = 1.0_f64;
        let clipped = clip_axis(u0, u1 - u0, (0.0, length), &mut t_enter, &mut t_exit)
            && clip_axis(
                v0,
                v1 - v0,
                (-dist_limit, dist_limit),
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
    (low, high): (f64, f64),
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
                (points, &predecessor_length),
                index,
                window * 0.5,
                true,
                looped,
            );
            let front = walk(
                (points, &predecessor_length),
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
    (points, predecessor_length): (&[(f64, f64)], &[f64]),
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
            let start = points[index];
            let delta = (points[neighbor].0 - start.0, points[neighbor].1 - start.1);
            let norm = delta.0.hypot(delta.1);
            // Eigen::MatrixBase::normalized preserves zero vectors; the partial branch
            // still assigns its sample (ExtrusionProcessor.hpp:395–400,415–420).
            let divisor = if norm > 0.0 { norm } else { 1.0 };
            position = (
                start.0 + remaining * delta.0 / divisor,
                start.1 + remaining * delta.1 / divisor,
            );
            break;
        }
        distance += line_dist;
        index = neighbor;
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

pub(super) fn signed_distance(
    tree: &LineDistanceTree<'_>,
    scale: CoordinateScale,
    point: (f64, f64),
) -> f64 {
    // `distance_from_lines<true>` on the previous-layer `lslices`
    // boundary (`SupportSpotsGenerator.cpp:153,178-180`): f64 unscaled
    // distance, negative inside. Kept in f64 — the `+ 0.5 * flow_width`
    // boundary check upstream (`0.5f * flow_width` cast to double)
    // decides sign on sub-ULP values for axis-aligned walls.
    let nearest = tree
        .nearest_mm([point.0, point.1], scale.factor())
        .expect("a nonempty boundary has a nearest line");
    tree.outside_mm([point.0, point.1], scale.factor()) as f64 * nearest.squared_distance.sqrt()
}
