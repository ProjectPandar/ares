// ShortestPath.cpp:103–137/164–176 at OrcaSlicer
// 345efec64c7b95a05ce5cf96df0dde66a158835c, specialized by chain_points.
use super::Point;

pub(super) fn chain_points_exact(points: &[Point]) -> Vec<usize> {
    let count = points.len();
    let endpoints = points
        .iter()
        .flat_map(|point| {
            let position = [point.x() as f64, point.y() as f64];
            [position, position]
        })
        .collect::<Vec<_>>();
    let states = (1 << count) * count * 2;
    let mut cost = vec![f64::MAX; states];
    let mut from = vec![usize::MAX; states];
    for last in 0..count {
        for orientation in 0..2 {
            cost[(((1 << last) * count) + last) * 2 + orientation] = 0.0;
        }
    }
    for mask in 1..1 << count {
        for state in mask * count * 2..(mask + 1) * count * 2 {
            if cost[state] < f64::MAX {
                extend_state(&endpoints, mask, state, (&mut cost, &mut from));
            }
        }
    }
    let full = (1 << count) - 1;
    let first_final = full * count * 2;
    let mut best = first_final;
    for state in first_final..states {
        if cost[state] < cost[best] {
            best = state;
        }
    }
    let mut output = vec![0; count];
    let mut state = best;
    for index in (0..count).rev() {
        output[index] = state / 2 % count;
        state = from[state];
    }
    output
}

// Keep the source candidate/orientation order and strict cost update. Even
// coincident endpoints retain their separate orientation/predecessor states.
fn extend_state(
    endpoints: &[[f64; 2]],
    mask: usize,
    state: usize,
    (cost, from): (&mut [f64], &mut [usize]),
) {
    let count = endpoints.len() / 2;
    for candidate in 0..count {
        if (mask >> candidate) & 1 != 0 {
            continue;
        }
        for orientation in 0..2 {
            let next = ((mask | (1 << candidate)) * count + candidate) * 2 + orientation;
            let entry = endpoints[2 * candidate + orientation];
            let exit = endpoints[2 * (state / 2 % count) + 1 - (state & 1)];
            let dx = entry[0] - exit[0];
            let dy = entry[1] - exit[1];
            let next_cost = cost[state] + (dx * dx + dy * dy).sqrt();
            if next_cost < cost[next] {
                cost[next] = next_cost;
                from[next] = state;
            }
        }
    }
}
