//! OrderingStrategies.cpp:30–102::tsp_2opt_improve (default ten passes),
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::{Point, position, squared};

struct Edges {
    lengths: Vec<f64>,
    squared: Vec<f64>,
}

impl Edges {
    fn new(path: &[usize], points: &[Point]) -> Self {
        let lengths = (0..path.len())
            .map(|i| {
                squared(
                    position(points[path[i]]),
                    position(points[path[(i + 1) % path.len()]]),
                )
                .sqrt()
            })
            .collect();
        let squared = (0..path.len())
            .map(|i| {
                squared(
                    position(points[path[i]]),
                    position(points[path[(i + 1) % path.len()]]),
                )
            })
            .collect();
        Self { lengths, squared }
    }
}

pub(super) fn improve(path: &mut [usize], points: &[Point]) -> bool {
    let count = path.len();
    let mut edges = Edges::new(path, points);
    let mut improved = false;
    for _ in 0..10 {
        let best = best_swap(path, points, &edges);
        if best.0 == count {
            break;
        }
        improved = true;
        path[best.0 + 1..best.1 + 1].reverse();
        edges = Edges::new(path, points);
    }
    improved
}

// The source best-gain scan, separated from its ten-pass/cache-refresh loop.
fn best_swap(path: &[usize], points: &[Point], edges: &Edges) -> (usize, usize) {
    let count = path.len();
    let mut best = (count, count);
    let mut best_gain = 0.0;
    for i in 0..count {
        let a = position(points[path[i]]);
        let next_a = position(points[path[(i + 1) % count]]);
        for j in i + 2..count {
            if i == 0 && j + 1 == count {
                continue;
            }
            let new_a = squared(position(points[path[j]]), a);
            let new_b = squared(position(points[path[(j + 1) % count]]), next_a);
            if new_a >= edges.squared[i] && new_b >= edges.squared[j] {
                continue;
            }
            let gain = edges.lengths[i] + edges.lengths[j] - new_a.sqrt() - new_b.sqrt();
            if gain > best_gain {
                best_gain = gain;
                best = (i, j);
            }
        }
    }
    best
}
