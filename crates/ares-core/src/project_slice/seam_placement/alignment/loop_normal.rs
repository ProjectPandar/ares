//! `place_on_loop` (`SeamPlacer.cpp:1498-1627`).

use super::{LayerSeamCandidates, Vec3, paint_rules::allowed_arc};
use crate::project_slice::seam_candidates::SeamCandidate;

pub(super) fn place_on_loop(layer: &LayerSeamCandidates, index: usize, target: Vec3) -> Vec3 {
    let selected = &layer.points[index];
    let perimeter = &layer.perimeters[selected.perimeter_index];
    let points = &layer.points[perimeter.start_index..perimeter.end_index];
    let count = points.len();
    let position = selected.position;
    if count < 2 {
        return Vec3::new(position.x, position.y, position.z);
    }
    let index = index - perimeter.start_index;
    let target = [f64::from(target.x), f64::from(target.y)];
    let mut best = None;
    let mut best_distance = f64::INFINITY;
    let mut arc = [0.0, 0.0];
    let mut cursor = [index, index];
    let mut best_arc = 0.0;
    let mut best_forward = true;
    for _ in 0..count {
        let forward = arc[1] <= arc[0];
        let direction = usize::from(forward);
        let i = cursor[direction];
        let j = step(i, count, forward);
        let (a, b) = if forward { (i, j) } else { (j, i) };
        let (solution, length) = consider_edge(points, (a, b), target, best_distance);
        if let Some((position, distance, fraction)) = solution {
            best = Some(position);
            best_distance = distance;
            best_forward = forward;
            best_arc = arc[direction] + (if forward { fraction } else { 1.0 - fraction }) * length;
        }
        arc[direction] += length;
        cursor[direction] = j;
    }
    let Some(mut result) = best else {
        return Vec3::new(position.x, position.y, position.z);
    };
    let Some(ends) = &perimeter.exact_paint_ends else {
        return Vec3::new(result[0] as f32, result[1] as f32, position.z);
    };
    let length = arc[0] + arc[1];
    let ahead = if best_forward {
        best_arc
    } else {
        length - best_arc
    };
    let (ahead_limit, ahead_end) = allowed_arc(points, index, true, ends.enforced);
    let (behind_limit, behind_end) = allowed_arc(points, index, false, ends.enforced);
    if ahead > ahead_limit && length - ahead > behind_limit {
        let to_result = |[x, y]: [f64; 2]| norm_squared([x - result[0], y - result[1]]);
        result = if to_result(ahead_end) <= to_result(behind_end) {
            ahead_end
        } else {
            behind_end
        };
    }
    Vec3::new(result[0] as f32, result[1] as f32, position.z)
}

type EdgeSolution = ([f64; 2], f64, f64);

fn consider_edge(
    points: &[SeamCandidate],
    (a, b): (usize, usize),
    target: [f64; 2],
    mut best_distance: f64,
) -> (Option<EdgeSolution>, f64) {
    let mut best = None;
    let origin = xy(&points[a]);
    let end = xy(&points[b]);
    let d = [end[0] - origin[0], end[1] - origin[1]];
    let length = norm_squared(d).sqrt();
    let g = [target[0] - origin[0], target[1] - origin[1]];
    let closest = if length > 0.0 {
        ((g[0] * d[0] + g[1] * d[1]) / (length * length)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if length > 0.0 && norm_squared([g[0] - closest * d[0], g[1] - closest * d[1]]) < best_distance
    {
        let mut na = vertex_normal(points, a);
        let mut nb = vertex_normal(points, b);
        if norm_squared(na) == 0.0 || norm_squared(nb) == 0.0 {
            na = edge_normal(origin, end);
            nb = na;
        }
        let m = [nb[0] - na[0], nb[1] - na[1]];
        let c0 = cross(g, na);
        let c1 = cross(g, m) - cross(d, na);
        let c2 = -cross(d, m);
        let mut consider = |u: f64| {
            if !(0.0..=1.0).contains(&u) {
                return;
            }
            let distance = norm_squared([g[0] - u * d[0], g[1] - u * d[1]]);
            if distance < best_distance {
                best = Some(([origin[0] + u * d[0], origin[1] + u * d[1]], distance, u));
                best_distance = distance;
            }
        };
        // Source absolute coefficient threshold, not a geometric tolerance.
        const EPS: f64 = 1e-12;
        if c2.abs() < EPS && c1.abs() < EPS {
            if c0.abs() < EPS {
                consider(closest);
            }
        } else if c2.abs() < EPS {
            consider(-c0 / c1);
        } else {
            let discriminant = c1 * c1 - 4.0 * c2 * c0;
            if discriminant >= 0.0 {
                let root = discriminant.sqrt();
                consider((-c1 - root) / (2.0 * c2));
                consider((-c1 + root) / (2.0 * c2));
            }
        }
    }
    (best, length)
}

fn vertex_normal(points: &[SeamCandidate], index: usize) -> [f64; 2] {
    let position = xy(&points[index]);
    let distinct = |forward| {
        let mut j = index;
        for _ in 1..points.len() {
            j = step(j, points.len(), forward);
            let neighbor = xy(&points[j]);
            if neighbor != position {
                return neighbor;
            }
        }
        position
    };
    let before = edge_normal(distinct(false), position);
    let after = edge_normal(position, distinct(true));
    let sum = [before[0] + after[0], before[1] + after[1]];
    let squared = norm_squared(sum);
    if squared > 0.0 {
        let length = squared.sqrt();
        [sum[0] / length, sum[1] / length]
    } else {
        [0.0, 0.0]
    }
}

fn edge_normal(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = norm_squared(d).sqrt();
    if length > 0.0 {
        [-d[1] / length, d[0] / length]
    } else {
        [0.0, 0.0]
    }
}

pub(super) fn xy(point: &SeamCandidate) -> [f64; 2] {
    [f64::from(point.position.x), f64::from(point.position.y)]
}

fn norm_squared([x, y]: [f64; 2]) -> f64 {
    x * x + y * y
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

pub(super) fn step(index: usize, count: usize, forward: bool) -> usize {
    if forward {
        if index + 1 == count { 0 } else { index + 1 }
    } else if index == 0 {
        count - 1
    } else {
        index - 1
    }
}
