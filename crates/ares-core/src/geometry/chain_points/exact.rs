// ShortestPath.cpp:103–137/164–176 at OrcaSlicer
// 345efec64c7b95a05ce5cf96df0dde66a158835c.

/// Paired entry/exit endpoints, with source orientation and optional start costs.
/// Callers supply one through twelve segments and, when present, one mask entry per segment.
pub(crate) fn chain_segments_exact(
    endpoints: &[[f64; 2]],
    could_reverse: Option<&[bool]>,
    start_near: Option<[f64; 2]>,
) -> Vec<(usize, bool)> {
    let count = endpoints.len() / 2;
    let states = (1 << count) * count * 2;
    let mut cost = vec![f64::MAX; states];
    let mut from = vec![usize::MAX; states];
    for last in 0..count {
        let reversible = could_reverse.is_none_or(|mask| mask[last]);
        for orientation in 0..=usize::from(reversible) {
            cost[(((1 << last) * count) + last) * 2 + orientation] = start_near
                .map_or(0.0, |start| {
                    distance(endpoints[2 * last + orientation], start)
                });
        }
    }
    for mask in 1..1 << count {
        for state in mask * count * 2..(mask + 1) * count * 2 {
            if cost[state] < f64::MAX {
                extend_state(
                    endpoints,
                    mask,
                    state,
                    could_reverse,
                    (&mut cost, &mut from),
                );
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
    let mut output = vec![(0, false); count];
    let mut state = best;
    for index in (0..count).rev() {
        output[index] = (state / 2 % count, (state & 1) != 0);
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
    could_reverse: Option<&[bool]>,
    (cost, from): (&mut [f64], &mut [usize]),
) {
    let count = endpoints.len() / 2;
    for candidate in 0..count {
        if (mask >> candidate) & 1 != 0 {
            continue;
        }
        let reversible = could_reverse.is_none_or(|mask| mask[candidate]);
        for orientation in 0..=usize::from(reversible) {
            let next = ((mask | (1 << candidate)) * count + candidate) * 2 + orientation;
            let entry = endpoints[2 * candidate + orientation];
            let exit = endpoints[2 * (state / 2 % count) + 1 - (state & 1)];
            let next_cost = cost[state] + distance(entry, exit);
            if next_cost < cost[next] {
                cost[next] = next_cost;
                from[next] = state;
            }
        }
    }
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}
