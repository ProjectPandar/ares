//! ToolOrderUtils.cpp:914–1141,1324–1494; MultiNozzleUtils.cpp:467–661.
use super::context::Context;
use std::collections::BTreeMap;

mod nozzles;

// Final2.5 FilamentGroup.cpp:103-112, shared with calc_one_change_budget.
pub(super) const FLUSH_VOLUME_TO_SCORE: f64 = 1.26 * 180.0 * 2.0 / 1000.0;

pub(super) fn score(context: &Context, map: &[usize]) -> f64 {
    let results = std::array::from_fn::<_, 2, _>(|head| score_nozzle(context, map, head));
    let flush = results.iter().map(|(cost, _)| *cost).sum::<i32>();
    let sequences = results.map(|(_, sequences)| sequences);
    let extruders = (0..2)
        .filter(|head| context.used.iter().any(|id| map[*id] == *head))
        .collect::<Vec<_>>();
    let mut last_head_index = 0;
    let mut filament_sequence = Vec::new();
    let mut previous_filament = None;
    for (layer, _) in context.layers.iter().enumerate() {
        let mut next_head = last_head_index;
        let mut append = |id: usize| {
            if previous_filament != Some(id) {
                filament_sequence.push(id);
                previous_filament = Some(id);
            }
        };
        if let Some(custom) = context.custom.get(&layer) {
            for &id in custom {
                append(id);
            }
            if let Some(&last) = custom.last() {
                // The single-nozzle source records the physical extruder index here.
                next_head = map[last];
            }
        } else {
            for offset in 0..extruders.len() {
                let head = extruders[(last_head_index + offset) % extruders.len()];
                let fragment = &sequences[head][layer];
                next_head = fragment.last().map_or(next_head, |_| head);
                fragment.iter().copied().for_each(&mut append);
            }
        }
        last_head_index = next_head;
    }
    // With one nozzle per head, a filament's AMS group is its assigned physical
    // head. Preload is absent in the static wrapper, so occupancy clearance in
    // simulate_filament_change_time is exactly this per-head loaded-filament state.
    let mut loaded = [None; 2];
    let mut time = 0.0;
    let [standard_load, selector_load] = context.load;
    let [standard_unload, selector_unload] = context.unload;
    let load = (standard_load - selector_load) + selector_load;
    let unload = selector_unload + (standard_unload - selector_unload);
    for id in filament_sequence {
        let head = map[id];
        if loaded[head] != Some(id) {
            if loaded[head].is_some() {
                time += f64::from(unload);
            }
            time += f64::from(load);
            loaded[head] = Some(id);
        }
    }
    // ToolOrdering.cpp:1532 supplies an empty print-time table under both time flags.
    // Actual change-time simulation above is unconditional, including flag zero.
    f64::from(flush) * FLUSH_VOLUME_TO_SCORE + time
}

// FilamentGroup.cpp:124–196; the static wrapper has no loaded status or preload.
pub(super) fn score_multi_nozzle(context: &Context, map: &[usize]) -> f64 {
    let heads = context
        .nozzle_groups
        .iter()
        .map(|nozzle| nozzle.extruder)
        .collect::<Vec<_>>();
    let (flush, sequences) = nozzles::sequences(context, map);
    let mut loaded = [None; 2];
    let mut time = 0.0;
    let [standard_load, selector_load] = context.load;
    let [standard_unload, selector_unload] = context.unload;
    let load = (standard_load - selector_load) + selector_load;
    let unload = selector_unload + (standard_unload - selector_unload);
    for id in sequences.into_iter().flatten() {
        let head = heads[map[id]];
        if loaded[head] != Some(id) {
            if loaded[head].is_some() {
                time += f64::from(unload);
            }
            time += f64::from(load);
            loaded[head] = Some(id);
        }
    }
    f64::from(flush) * FLUSH_VOLUME_TO_SCORE + time
}

fn score_nozzle(context: &Context, map: &[usize], head: usize) -> (i32, Vec<Vec<usize>>) {
    let mut output = Vec::with_capacity(context.layers.len());
    // Ordinary maps use physical heads; multi-nozzle maps use nozzle-group IDs.
    let matrix = if context.manual_multi_nozzle {
        &context.matrices[context.nozzle_groups[head].extruder]
    } else {
        &context.matrices[head]
    };
    let mut previous: Option<usize> = None;
    let mut cache = BTreeMap::<u128, (f32, Vec<usize>)>::new();
    let mut cost = 0_i32;
    for (index, layer) in context.layers.iter().enumerate() {
        let current = layer
            .iter()
            .copied()
            .filter(|id| map[*id] == head)
            .collect::<Vec<_>>();
        let sequence = if let Some(custom) = context
            .custom
            .get(&index)
            .filter(|_| !context.manual_multi_nozzle)
        {
            let sequence = custom
                .iter()
                .copied()
                .filter(|id| map[*id] == head)
                .collect::<Vec<_>>();
            cost = custom_cost(matrix, &sequence, previous, cost);
            sequence
        } else {
            // Source uint128_t key; inter-layer forecast is disabled.
            let mut key = current.iter().fold(0_u128, |bits, id| {
                bits | 1_u128.checked_shl(*id as u32).unwrap_or(0)
            });
            if let Some(id) = previous {
                key |= 1_u128.checked_shl((64 + id) as u32).unwrap_or(0);
            }
            let (volume, sequence) = cache
                .entry(key)
                .or_insert_with(|| order(matrix, &current, previous));
            cost = (cost as f32 + *volume) as i32;
            sequence.clone()
        };
        if let Some(&last) = sequence.last() {
            previous = Some(last);
        }
        output.push(sequence);
    }
    (cost, output)
}

fn custom_cost(
    matrix: &[Vec<f32>],
    sequence: &[usize],
    mut previous: Option<usize>,
    mut cost: i32,
) -> i32 {
    for &id in sequence {
        if let Some(from) = previous {
            cost = (cost as f32 + matrix[from][id]) as i32;
        }
        previous = Some(id);
    }
    cost
}

fn relax_state(
    tables: (&mut [f32], &mut [Option<usize>]),
    state: usize,
    all: &[usize],
    matrix: &[Vec<f32>],
) {
    let (cache, prev) = tables;
    let size = all.len();
    for target in 0..size {
        if state >> target & 1 == 0 {
            continue;
        }
        for middle in 0..size {
            if state >> middle & 1 == 0 {
                continue;
            }
            let candidate =
                cache[(state - (1 << target)) * size + middle] + matrix[all[middle]][all[target]];
            let slot = state * size + target;
            if cache[slot] > candidate {
                cache[slot] = candidate;
                prev[slot] = Some(middle);
            }
        }
    }
}

// get_extruders_order / solve_extruder_order: bounded enumeration admits at most
// thirteen used filaments, so the source <=20 Hamiltonian-DP branch always applies.
fn order(matrix: &[Vec<f32>], current: &[usize], start: Option<usize>) -> (f32, Vec<usize>) {
    if current.is_empty() {
        return (0.0, Vec::new());
    }
    if current.len() == 1 {
        return (
            start.map_or(0.0, |id| matrix[id][current[0]]),
            current.to_vec(),
        );
    }
    let mut all = current.to_vec();
    let mut added = false;
    if let Some(start) = start {
        if let Some(position) = all.iter().position(|id| *id == start) {
            all.swap(0, position);
        } else {
            all.insert(0, start);
            added = true;
        }
    }
    let start = start.unwrap_or(all[0]);
    let size = all.len();
    let states = 1 << size;
    let mut cache = vec![f32::MAX; states * size];
    let mut prev = vec![None; states * size];
    cache[size] = 0.0;
    for state in 0..states {
        if state & 1 == 0 {
            continue;
        }
        relax_state((&mut cache, &mut prev), state, &all, matrix);
    }
    let final_state = states - 1;
    let mut cost = f32::MAX;
    let mut destination = 0;
    for target in 0..size {
        if all[target] != start && cost > cache[final_state * size + target] {
            cost = cache[final_state * size + target];
            destination = target;
        }
    }
    let mut path = Vec::with_capacity(size);
    let mut state = final_state;
    let mut current = Some(destination);
    while let Some(point) = current {
        path.push(all[point]);
        current = prev[state * size + point];
        state -= 1 << point;
    }
    if added {
        path.pop();
    }
    path.reverse();
    (cost, path)
}
