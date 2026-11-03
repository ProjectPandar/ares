//! Convex helpers of the print-volume bed shape classification.

use super::SCENE_EPSILON;

/// 4 corners on the bbox edges (axis-aligned rectangle).
pub(super) fn is_axis_aligned_rectangle(area: &[crate::Point2d]) -> bool {
    let min = [
        area.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
        area.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
    ];
    let max = [
        area.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
        area.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
    ];
    area.iter().all(|p| {
        let on_x = (p.x - min[0]).abs() < SCENE_EPSILON || (p.x - max[0]).abs() < SCENE_EPSILON;
        let on_y = (p.y - min[1]).abs() < SCENE_EPSILON || (p.y - max[1]).abs() < SCENE_EPSILON;
        on_x && on_y
    })
}

/// Andrew's monotone chain convex hull.
pub(super) fn convex_hull(points: &[crate::Point2d]) -> Vec<[f64; 2]> {
    let mut pts: Vec<[f64; 2]> = points.iter().map(|p| [p.x, p.y]).collect();
    pts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Point-in-convex-polygon (CCW hull).
pub(super) fn point_in_convex(hull: &[[f64; 2]], p: [f64; 2]) -> bool {
    (0..hull.len()).all(|i| {
        let a = hull[i];
        let b = hull[(i + 1) % hull.len()];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        cross >= 0.0
    })
}
