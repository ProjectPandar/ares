//! `GCode.cpp:594–710`: shorter squared-length route around the tower bbox.
use crate::geometry::CoordinateScale;
type Point = [i64; 2];

/// `WipeTowerIntegration::generate_path_to_wipe_tower`: the route from `start` to `end` around
/// the `[low, high]` avoid box inflated by the routing clearance, inside the `[min, max]` bed.
pub(super) fn path_to_wipe_tower(
    start: Point,
    end_local: Point,
    [mut low, mut high]: [Point; 2],
    [bed_min, bed_max]: [Point; 2],
    scale: CoordinateScale,
) -> Vec<Point> {
    let offset = (2.0 / scale.factor()) as i64;
    for a in 0..2 {
        low[a] -= offset;
        high[a] += offset;
    }
    let points = [low, [high[0], low[1]], high, [low[0], high[1]]];
    if points
        .iter()
        .any(|p| (0..2).any(|a| p[a] <= bed_min[a] || p[a] >= bed_max[a]))
    {
        return vec![end_local];
    }
    let direction = if (end_local[0] - low[0]).abs() < (high[0] - low[0]) / 2 {
        -1.0
    } else {
        1.0
    };
    let mm = |p: Point| [scale.unscale(p[0]), scale.unscale(p[1])];
    let intersect = |a: [f64; 2], v: [f64; 2], b: [f64; 2], c: [f64; 2], ray: bool| {
        let w = [c[0] - b[0], c[1] - b[1]];
        let d = [a[0] - b[0], a[1] - b[1]];
        let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
        let denominator = cross(v, w);
        if denominator.abs() < 1e-4 {
            return None;
        }
        let t = cross(w, d) / denominator;
        let u = cross(v, d) / denominator;
        (t >= 0.0 && (ray || t <= 1.0) && (0.0..=1.0).contains(&u)).then(|| {
            [
                ((a[0] + t * v[0]) / scale.factor()) as i64,
                ((a[1] + t * v[1]) / scale.factor()) as i64,
            ]
        })
    };
    let Some((end_edge, hit)) = (0..4).find_map(|i| {
        intersect(
            mm(end_local),
            [direction, 0.0],
            mm(points[i]),
            mm(points[(i + 1) % 4]),
            true,
        )
        .map(|p| (i, p))
    }) else {
        return vec![end_local];
    };
    let a = mm(start);
    let b = mm(hit);
    let v = [b[0] - a[0], b[1] - a[1]];
    let entry = (0..4).filter(|&i| i != end_edge).find_map(|i| {
        intersect(a, v, mm(points[i]), mm(points[(i + 1) % 4]), false).map(|p| (i, p))
    });
    let mut path = if let Some((entry_edge, entry)) = entry {
        let around = |forward: bool| {
            let mut i = if forward {
                (entry_edge + 1) % 4
            } else {
                entry_edge
            };
            let end = if forward {
                end_edge
            } else {
                (end_edge + 1) % 4
            };
            let squared = |a: Point, b: Point| {
                let a = mm(a);
                let b = mm(b);
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
            };
            let mut length = squared(entry, points[i]);
            let mut path = vec![entry];
            let step = if forward { 1 } else { 3 };
            while i != end {
                let next = (i + step) % 4;
                length += squared(points[i], points[next]);
                path.push(points[i]);
                i = next;
            }
            path.push(points[end]);
            path.push(hit);
            length += squared(hit, points[end]);
            (path, length)
        };
        let (forward, a) = around(true);
        let (backward, b) = around(false);
        if a < b { forward } else { backward }
    } else {
        vec![hit]
    };
    path.push(end_local);
    path
}
