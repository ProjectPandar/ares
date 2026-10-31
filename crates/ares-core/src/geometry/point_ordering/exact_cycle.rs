//! OrderingStrategies.cpp:230–266::tsp_exact_cycle at
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::{Point, distance};

pub(super) fn tsp_exact_cycle(points: &[Point]) -> Vec<usize> {
    let mut path = (0..points.len()).collect::<Vec<_>>();
    if points.len() <= 3 {
        return path;
    }
    let count = points.len() - 1;
    let mut cost = vec![f64::MAX; (1 << count) * count];
    let mut from = vec![usize::MAX; cost.len()];
    for last in 0..count {
        cost[(1 << last) * count + last] = distance(points[0], points[last + 1]);
    }
    for mask in 1..1 << count {
        for last in 0..count {
            if cost[mask * count + last] < f64::MAX {
                extend(mask, last, points, &mut cost, &mut from);
            }
        }
    }
    let full = (1 << count) - 1;
    let mut best = full * count;
    for last in 1..count {
        if cost[full * count + last] + distance(points[last + 1], points[0])
            < cost[best] + distance(points[best % count + 1], points[0])
        {
            best = full * count + last;
        }
    }
    let mut state = best;
    for index in (1..points.len()).rev() {
        path[index] = state % count + 1;
        state = from[state];
    }
    path
}

fn extend(mask: usize, last: usize, points: &[Point], cost: &mut [f64], from: &mut [usize]) {
    let count = points.len() - 1;
    let state = mask * count + last;
    let current = cost[state];
    for candidate in 0..count {
        if mask >> candidate & 1 == 0 {
            let next = (mask | 1 << candidate) * count + candidate;
            let next_cost = current + distance(points[last + 1], points[candidate + 1]);
            if next_cost < cost[next] {
                cost[next] = next_cost;
                from[next] = state;
            }
        }
    }
}
