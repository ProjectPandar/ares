//! `ToolOrderUtils.cpp` greedy, Hamilton path and one-layer forecast.

pub(super) fn order(
    matrix: &[Vec<f32>],
    current: &[usize],
    next: &[usize],
    start: Option<usize>,
    forecast: bool,
) -> Vec<usize> {
    if current.len() <= 1 {
        return current.to_vec();
    }
    if forecast {
        forecast_order(matrix, current, next, start)
    } else if current.len() <= 20 {
        hamilton(matrix, current, start)
    } else {
        greedy(matrix, current, start)
    }
}

fn greedy(matrix: &[Vec<f32>], current: &[usize], mut previous: Option<usize>) -> Vec<usize> {
    let mut visited = vec![false; current.len()];
    let mut sequence = Vec::with_capacity(current.len());
    while sequence.len() < current.len() {
        let prior = *previous
            .get_or_insert_with(|| current[visited.iter().position(|&used| !used).unwrap()]);
        let mut target = None;
        let mut cost = i32::MAX;
        for index in (0..current.len()).filter(|&index| !visited[index]) {
            let candidate = matrix[prior][current[index]];
            if candidate < cost as f32 || (candidate == cost as f32 && prior == current[index]) {
                target = Some(index);
                // The upstream greedy comparison intentionally truncates to int.
                cost = candidate as i32;
            }
        }
        let target = target.expect("greedy source search selects an unvisited filament");
        sequence.push(current[target]);
        previous = Some(current[target]);
        visited[target] = true;
    }
    sequence
}

fn hamilton(matrix: &[Vec<f32>], current: &[usize], start: Option<usize>) -> Vec<usize> {
    let mut tools = current.to_vec();
    let mut inserted = false;
    let first = if let Some(start) = start {
        if let Some(index) = tools.iter().position(|&id| id == start) {
            tools.swap(0, index);
        } else {
            tools.insert(0, start);
            inserted = true;
        }
        start
    } else {
        tools[0]
    };
    let count = tools.len();
    let states = 1usize << count;
    let mut cache = vec![0x7fff_ffffu32 as f32; states * count];
    let mut previous = vec![-1i32; states * count];
    cache[count] = 0.0;
    for state in (0..states).filter(|state| state & 1 != 0) {
        relax(matrix, &tools, state, &mut cache, &mut previous);
    }
    let mut cost = f32::MAX;
    let mut destination = 0;
    for (index, _) in tools.iter().enumerate().filter(|&(_, &tool)| tool != first) {
        if cost > cache[(states - 1) * count + index] {
            cost = cache[(states - 1) * count + index];
            destination = index;
        }
    }
    let mut sequence = Vec::with_capacity(count);
    let mut state = states - 1;
    let mut point = destination as i32;
    while point != -1 {
        let index = point as usize;
        sequence.push(tools[index]);
        point = previous[state * count + index];
        state -= 1 << index;
    }
    if inserted {
        sequence.pop();
    }
    sequence.reverse();
    sequence
}

fn relax(
    matrix: &[Vec<f32>],
    tools: &[usize],
    state: usize,
    cache: &mut [f32],
    previous: &mut [i32],
) {
    let count = tools.len();
    for target in (0..count).filter(|&index| state >> index & 1 != 0) {
        for midpoint in (0..count).filter(|&index| state >> index & 1 != 0) {
            let candidate = cache[(state - (1 << target)) * count + midpoint]
                + matrix[tools[midpoint]][tools[target]];
            let slot = state * count + target;
            if cache[slot] > candidate {
                cache[slot] = candidate;
                previous[slot] = midpoint as i32;
            }
        }
    }
}

fn forecast_order(
    matrix: &[Vec<f32>],
    current: &[usize],
    next: &[usize],
    start: Option<usize>,
) -> Vec<usize> {
    let mut current = current.to_vec();
    let mut next = next.to_vec();
    current.sort_unstable();
    next.sort_unstable();
    let mut best_cost = f32::MAX;
    let mut best_changes = usize::MAX;
    let mut best = Vec::new();
    loop {
        let (cost, previous, changes) = sequence_cost(matrix, &current, start, 0.0);
        if cost > best_cost {
            if !next_permutation(&mut current) {
                break;
            }
            continue;
        }
        loop {
            let (total, _, next_changes) = sequence_cost(matrix, &next, previous, cost);
            let changes = changes + next_changes;
            if total < best_cost || (total == best_cost && changes < best_changes) {
                best_cost = total;
                best_changes = changes;
                best.clone_from(&current);
            }
            if !next_permutation(&mut next) {
                break;
            }
        }
        if !next_permutation(&mut current) {
            break;
        }
    }
    best
}

fn sequence_cost(
    matrix: &[Vec<f32>],
    sequence: &[usize],
    mut previous: Option<usize>,
    mut cost: f32,
) -> (f32, Option<usize>, usize) {
    let mut changes = 0;
    for &id in sequence {
        if let Some(previous) = previous {
            cost += matrix[previous][id];
            changes += usize::from(previous != id);
        }
        previous = Some(id);
    }
    (cost, previous, changes)
}

fn next_permutation(sequence: &mut [usize]) -> bool {
    let Some(pivot) = (1..sequence.len()).rfind(|&index| sequence[index - 1] < sequence[index])
    else {
        sequence.reverse();
        return false;
    };
    let successor = (pivot..sequence.len())
        .rfind(|&index| sequence[pivot - 1] < sequence[index])
        .unwrap();
    sequence.swap(pivot - 1, successor);
    sequence[pivot..].reverse();
    true
}
